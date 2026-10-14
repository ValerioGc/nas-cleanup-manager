//! The door to the launcher, `NAS_LAUNCHER_SOCKET`: the launcher knows this application by its
//! cgroup and lets it write only as itself. Through it the application measures and cleans the
//! system (`POST /api/apps/self/cleanup`, allowed by `permissions.cleanup`), reads and empties the
//! caches of the NAS, says whether it has scheduled cleanups to run without its window, shows a
//! cleanup among the operations of the NAS and says when one ended.
//!
//! Plain HTTP/1.1 over the unix socket, one request per connection. Without the socket (in
//! development) nothing can be measured or cleaned, and the answers say so.

use serde_json::{json, Value};
use std::{path::PathBuf, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

pub const ID: &str = "nas-cleanup-manager";
const TIMEOUT: Duration = Duration::from_secs(10);
/// A cleanup waits for apt: the launcher gives it two hours, and so does this side.
const CLEANUP_TIMEOUT: Duration = Duration::from_secs(2 * 60 * 60 + 60);

#[derive(Debug, Clone, Default)]
pub struct Launcher {
    socket: Option<PathBuf>,
}

/// An answer of the launcher: its status and its body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    pub status: u16,
    pub body: String,
}

impl Launcher {
    pub fn new(socket: Option<PathBuf>) -> Self {
        Self { socket }
    }

    pub fn from_env() -> Self {
        Self::new(
            std::env::var_os("NAS_LAUNCHER_SOCKET")
                .filter(|value| !value.is_empty())
                .map(PathBuf::from),
        )
    }

    pub(crate) async fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<&Value>,
        timeout: Duration,
    ) -> Result<Answer, String> {
        let socket = self
            .socket
            .as_ref()
            .ok_or("the launcher is not reachable")?;
        let body = body.map(Value::to_string).unwrap_or_default();
        let exchange = async {
            let mut stream = UnixStream::connect(socket)
                .await
                .map_err(|error| format!("the launcher is not reachable: {error}"))?;
            let request = format!(
                "{method} {path} HTTP/1.1\r\nHost: nas\r\nConnection: close\r\n\
                 Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            );
            stream
                .write_all(request.as_bytes())
                .await
                .map_err(|error| format!("the launcher did not take the request: {error}"))?;
            let mut raw = Vec::new();
            stream
                .read_to_end(&mut raw)
                .await
                .map_err(|error| format!("the launcher did not answer: {error}"))?;
            parse(&raw)
        };
        tokio::time::timeout(timeout, exchange)
            .await
            .map_err(|_| "the launcher did not answer in time".to_owned())?
    }

    /// The body of a successful answer as JSON, or why there is none: the reason the launcher gave.
    async fn json(
        &self,
        method: &str,
        path: &str,
        body: Option<&Value>,
        timeout: Duration,
    ) -> Result<Value, String> {
        let answer = self.request(method, path, body, timeout).await?;
        if answer.status >= 300 {
            let reason = serde_json::from_str::<Value>(&answer.body)
                .ok()
                .and_then(|value| value["error"].as_str().map(str::to_owned))
                .unwrap_or(answer.body);
            return Err(if reason.trim().is_empty() {
                format!("the launcher answered {}", answer.status)
            } else {
                reason
            });
        }
        if answer.body.trim().is_empty() {
            return Ok(Value::Null);
        }
        serde_json::from_str(&answer.body).map_err(|error| error.to_string())
    }

    /// An overview, a preview or a cleanup of one item of the launcher's fixed list.
    pub async fn cleanup(&self, request: &Value) -> Result<Value, String> {
        self.json(
            "POST",
            "/api/apps/self/cleanup",
            Some(request),
            CLEANUP_TIMEOUT,
        )
        .await
    }

    /// The caches of the NAS: `[{ app, id, label, bytes, items }]`.
    pub async fn caches(&self) -> Result<Value, String> {
        self.json("GET", "/api/caches", None, CLEANUP_TIMEOUT).await
    }

    pub async fn clear_cache(&self, app: &str, id: &str) -> Result<(), String> {
        self.json(
            "DELETE",
            &format!("/api/caches/{app}/{id}"),
            None,
            CLEANUP_TIMEOUT,
        )
        .await
        .map(drop)
    }

    /// A request whose answer matters only to the journal: nothing without the socket.
    async fn send(&self, method: &str, path: &str, body: &Value) {
        if self.socket.is_none() {
            return;
        }
        if let Err(reason) = self.json(method, path, Some(body), TIMEOUT).await {
            eprintln!("<4>{method} {path}: {reason}");
        }
    }

    /// Says whether this application has work to do without its window.
    pub async fn background(&self, needed: bool) {
        self.send(
            "PUT",
            "/api/apps/self/background",
            &json!({ "needed": needed }),
        )
        .await;
    }

    /// Shows a cleanup among the operations of the NAS: it cannot be paused or stopped halfway
    /// through an item, which apt would leave unfinished.
    pub async fn start_task(&self, id: &str, name: &str) {
        let task = json!({
            "id": id,
            "source": ID,
            "name": name,
            "progress": 0,
            "pausable": false,
            "cancellable": false,
        });
        self.send("POST", "/api/tasks", &task).await;
    }

    /// Reports where a cleanup is, or closes it with `state`.
    pub async fn update_task(&self, id: &str, update: &Value) {
        self.send("PATCH", &format!("/api/tasks/{id}"), update)
            .await;
    }

    /// A notification under the name of this application.
    pub async fn notify(&self, title: &str, message: &str, tone: &str) {
        let notification = json!({
            "title": title,
            "message": message,
            "source": ID,
            "tone": tone,
        });
        self.send("POST", "/api/notifications", &notification).await;
    }
}

/// The status and the body of a response, chunked or not.
pub fn parse(raw: &[u8]) -> Result<Answer, String> {
    let end = raw
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or("the launcher answered something that is not HTTP")?;
    let head = String::from_utf8_lossy(&raw[..end]);
    let body = &raw[end + 4..];
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse().ok())
        .ok_or("the launcher answered without a status")?;
    let chunked = head.lines().any(|line| {
        line.to_ascii_lowercase()
            .starts_with("transfer-encoding: chunked")
    });
    let body = if chunked {
        dechunk(body)
    } else {
        body.to_vec()
    };
    Ok(Answer {
        status,
        body: String::from_utf8_lossy(&body).into_owned(),
    })
}

fn dechunk(mut body: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    while let Some(line) = body.windows(2).position(|window| window == b"\r\n") {
        let size = std::str::from_utf8(&body[..line])
            .ok()
            .and_then(|size| usize::from_str_radix(size.trim(), 16).ok());
        let rest = &body[line + 2..];
        match size {
            Some(size) if size > 0 && rest.len() >= size => {
                out.extend_from_slice(&rest[..size]);
                body = rest[size..].strip_prefix(b"\r\n").unwrap_or(&rest[size..]);
            }
            _ => break,
        }
    }
    out
}

#[cfg(test)]
#[path = "../../tests/unit/axum/launcher.rs"]
mod tests;
