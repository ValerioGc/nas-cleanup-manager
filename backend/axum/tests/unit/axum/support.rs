use crate::{launcher::Launcher, MIGRATOR};
use serde_json::{json, Value};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
};

pub async fn database() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("database");
    MIGRATOR.run(&pool).await.expect("migrations");
    pool
}

#[derive(Clone)]
pub struct FakeLauncher {
    pub socket: PathBuf,
    pub requests: Arc<Mutex<Vec<(String, Value)>>>,
}

impl FakeLauncher {
    pub fn launcher(&self) -> Launcher {
        Launcher::new(Some(self.socket.clone()))
    }

    pub fn asked(&self) -> Vec<String> {
        self.requests
            .lock()
            .expect("requests")
            .iter()
            .map(|(line, _)| line.clone())
            .collect()
    }

    pub fn bodies(&self, line: &str) -> Vec<Value> {
        self.requests
            .lock()
            .expect("requests")
            .iter()
            .filter(|(asked, _)| asked == line)
            .map(|(_, body)| body.clone())
            .collect()
    }
}

fn answer(line: &str, body: &Value) -> (u16, Value) {
    match line {
        "POST /api/apps/self/cleanup" => match (body["action"].as_str(), body["item"].as_str()) {
            (Some("overview"), _) => (
                200,
                json!({
                    "total": 1000, "free": 400,
                    "categories": [{ "id": "logs", "bytes": 100 }],
                    "disks": [{ "name": "Dati", "total": 2000, "free": 1500 }],
                }),
            ),
            (_, Some("journal")) => (503, json!({ "error": "journalctl failed" })),
            (Some(action), Some(item)) => (
                200,
                json!({
                    "item": item,
                    "bytes": if action == "clean" { 70 } else { 100 },
                    "entries": ["/var/log/syslog.1"],
                    "more": 0,
                }),
            ),
            _ => (
                400,
                json!({ "error": "a preview or a cleanup names its item" }),
            ),
        },
        "GET /api/caches" => (
            200,
            json!([
                { "app": "nas-file-manager", "id": "previews", "label": "Miniature",
                  "bytes": 1000, "items": 3 },
                { "app": "nas-file-manager", "id": "trash", "label": "Cestino di Explorer",
                  "bytes": 500, "items": 2, "kind": "trash" }
            ]),
        ),
        _ => (204, Value::Null),
    }
}

async fn read_request(stream: &mut UnixStream) -> Option<(String, Value)> {
    let mut raw = Vec::new();
    let mut buffer = [0; 4096];
    let (head, length) = loop {
        let read = stream.read(&mut buffer).await.unwrap_or(0);
        if read == 0 {
            return None;
        }
        raw.extend_from_slice(&buffer[..read]);
        if let Some(end) = raw.windows(4).position(|window| window == b"\r\n\r\n") {
            let head = String::from_utf8_lossy(&raw[..end]).into_owned();
            let length = head
                .lines()
                .find_map(|line| line.strip_prefix("Content-Length: "))
                .and_then(|length| length.trim().parse::<usize>().ok())
                .unwrap_or(0);
            raw.drain(..end + 4);
            break (head, length);
        }
    };
    while raw.len() < length {
        let read = stream.read(&mut buffer).await.unwrap_or(0);
        if read == 0 {
            break;
        }
        raw.extend_from_slice(&buffer[..read]);
    }
    let line = head
        .lines()
        .next()
        .unwrap_or_default()
        .trim_end_matches(" HTTP/1.1")
        .to_owned();
    let body: Value = serde_json::from_slice(&raw).unwrap_or(Value::Null);
    Some((line, body))
}

pub async fn fake_launcher() -> FakeLauncher {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let socket = std::env::temp_dir().join(format!(
        "nas-cleanup-launcher-{}-{}.sock",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_file(&socket);
    let listener = UnixListener::bind(&socket).expect("socket");
    let fake = FakeLauncher {
        socket,
        requests: Arc::default(),
    };
    let requests = Arc::clone(&fake.requests);
    tokio::spawn(async move {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                return;
            };
            let requests = Arc::clone(&requests);
            tokio::spawn(async move {
                let Some((line, body)) = read_request(&mut stream).await else {
                    return;
                };
                let (status, reply) = answer(&line, &body);
                requests.lock().expect("requests").push((line, body));
                let reply = if reply.is_null() {
                    String::new()
                } else {
                    reply.to_string()
                };
                let _ = stream
                    .write_all(
                        format!(
                            "HTTP/1.1 {status} X\r\nContent-Length: {}\r\n\r\n{reply}",
                            reply.len()
                        )
                        .as_bytes(),
                    )
                    .await;
            });
        }
    });
    fake
}
