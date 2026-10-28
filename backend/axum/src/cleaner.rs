//! Measures and cleans items through the launcher. System items run in its root helper and caches
//! go through the apps that own them. Only one cleanup runs at a time. Each run is shown as a
//! launcher task, recorded in the history and optionally followed by a notification.

use crate::{
    catalog,
    launcher::Launcher,
    store::{self, Cleanup, ItemResult, Settings},
};
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::Notify;

const TASK: &str = "nas-cleanup-manager-run";

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |time| {
            i64::try_from(time.as_millis()).unwrap_or(i64::MAX)
        })
}

/// Progress of the running cleanup.
#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub running: bool,
    pub rule: Option<String>,
    pub item: Option<String>,
    pub done: usize,
    pub total: usize,
}

pub struct Cleaner {
    pool: SqlitePool,
    launcher: Launcher,
    progress: Mutex<Progress>,
    /// Next run of each enabled rule, kept by the scheduler.
    next: Mutex<HashMap<String, i64>>,
    rules_changed: Notify,
}

/// Formats a size for notifications, for example `1.5 GB`.
pub fn size_text(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

impl Cleaner {
    pub fn new(pool: SqlitePool, launcher: Launcher) -> Self {
        Self {
            pool,
            launcher,
            progress: Mutex::default(),
            next: Mutex::default(),
            rules_changed: Notify::new(),
        }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub fn launcher(&self) -> &Launcher {
        &self.launcher
    }

    pub fn progress(&self) -> Progress {
        self.progress
            .lock()
            .map(|progress| progress.clone())
            .unwrap_or_default()
    }

    fn set_progress(&self, change: impl FnOnce(&mut Progress)) {
        if let Ok(mut progress) = self.progress.lock() {
            change(&mut progress);
        }
    }

    pub fn next_run(&self, id: &str) -> Option<i64> {
        self.next.lock().ok()?.get(id).copied()
    }

    pub fn set_next(&self, id: &str, at: Option<i64>) {
        if let Ok(mut next) = self.next.lock() {
            match at {
                Some(at) => next.insert(id.to_owned(), at),
                None => next.remove(id),
            };
        }
    }

    /// Drops the next run of rules that are no longer scheduled.
    pub fn keep_next(&self, ids: &[String]) {
        if let Ok(mut next) = self.next.lock() {
            next.retain(|id, _| ids.contains(id));
        }
    }

    pub fn forget_next(&self) {
        if let Ok(mut next) = self.next.lock() {
            next.clear();
        }
    }

    /// Wakes the scheduler and updates the background declaration.
    pub async fn rules_changed(&self) {
        self.rules_changed.notify_one();
        self.declare_background().await;
    }

    pub async fn schedule_changed(&self) {
        self.rules_changed.notified().await;
    }

    /// Asks the launcher to keep the app running while any rule is enabled.
    pub async fn declare_background(&self) {
        let needed = store::rules(&self.pool)
            .await
            .map(|rules| rules.iter().any(|rule| rule.enabled))
            .unwrap_or(false);
        self.launcher.background(needed).await;
    }

    fn request(action: &str, item: &str, settings: Settings) -> Value {
        json!({
            "action": action,
            "item": item,
            "options": {
                "temporaryDays": settings.temporary_days,
                "journalMb": settings.journal_mb,
            },
        })
    }

    /// Measures the space an item currently holds.
    async fn measure(&self, item: &str, settings: Settings) -> ItemResult {
        let measured = if let Some((app, id)) = catalog::cache_of(item) {
            catalog::caches(&self.launcher).await.map(|caches| {
                let bytes = caches
                    .iter()
                    .find(|cache| cache.app == app && cache.id == id)
                    .map_or(0, |cache| cache.bytes);
                ItemResult {
                    id: item.to_owned(),
                    bytes,
                    ..ItemResult::default()
                }
            })
        } else {
            self.launcher
                .cleanup(&Self::request("preview", item, settings))
                .await
                .and_then(|value| outcome(item, value))
        };
        measured.unwrap_or_else(|error| failed(item, error))
    }

    /// Cleans an item and returns the space freed.
    async fn free(&self, item: &str, settings: Settings) -> ItemResult {
        let freed = if let Some((app, id)) = catalog::cache_of(item) {
            let before = self.measure(item, settings).await;
            match self.launcher.clear_cache(app, id).await {
                Ok(()) => Ok(ItemResult {
                    bytes: before.bytes,
                    ..before
                }),
                Err(error) => Err(error),
            }
        } else {
            self.launcher
                .cleanup(&Self::request("clean", item, settings))
                .await
                .and_then(|value| outcome(item, value))
        };
        freed.unwrap_or_else(|error| failed(item, error))
    }

    /// Measures every item without deleting anything.
    pub async fn preview(&self, items: &[String]) -> Result<Vec<ItemResult>, sqlx::Error> {
        let settings = store::settings(&self.pool).await?;
        let mut results = Vec::with_capacity(items.len());
        for item in items {
            results.push(self.measure(item, settings).await);
        }
        Ok(results)
    }

    /// System disk usage with the cache total and the last cleanup.
    pub async fn overview(&self) -> Result<Value, String> {
        let mut overview = self
            .launcher
            .cleanup(&json!({ "action": "overview" }))
            .await?;
        let caches: u64 = catalog::caches(&self.launcher)
            .await
            .unwrap_or_default()
            .iter()
            .filter(|cache| !cache.is_trash())
            .map(|cache| cache.bytes)
            .sum();
        if let Some(categories) = overview["categories"].as_array_mut() {
            categories.push(json!({ "id": "caches", "bytes": caches }));
        }
        let last = store::history(&self.pool)
            .await
            .ok()
            .and_then(|history| history.into_iter().next());
        overview["last"] = serde_json::to_value(last).unwrap_or(Value::Null);
        Ok(overview)
    }

    /// Starts a cleanup in the background. Returns false when one is already running.
    pub fn start(self: &Arc<Self>, items: Vec<String>, rule: Option<String>) -> bool {
        let started = self
            .progress
            .lock()
            .map(|mut progress| {
                if progress.running {
                    return false;
                }
                *progress = Progress {
                    running: true,
                    rule: rule.clone(),
                    item: None,
                    done: 0,
                    total: items.len(),
                };
                true
            })
            .unwrap_or(false);
        if started {
            let cleaner = Arc::clone(self);
            tokio::spawn(async move {
                cleaner.run(items, rule).await;
            });
        }
        started
    }

    /// Cleans the items in order and records the result.
    pub async fn run(&self, items: Vec<String>, rule: Option<String>) -> Option<Cleanup> {
        let settings = store::settings(&self.pool).await.unwrap_or_default();
        let name = rule.as_ref().map_or_else(
            || "Pulizia".to_owned(),
            |rule| format!("Pulizia pianificata ({rule})"),
        );
        self.launcher.start_task(TASK, &name).await;
        let total = items.len().max(1);
        let mut results = Vec::with_capacity(items.len());
        for (done, item) in items.iter().enumerate() {
            self.set_progress(|progress| {
                progress.item = Some(item.clone());
                progress.done = done;
            });
            self.launcher
                .update_task(
                    TASK,
                    &json!({ "progress": done * 100 / total, "detail": item }),
                )
                .await;
            results.push(self.free(item, settings).await);
        }
        let failed = results
            .iter()
            .filter(|result| result.error.is_some())
            .count();
        let recorded = store::record(&self.pool, now_ms(), rule.as_deref(), &results)
            .await
            .map_err(|error| eprintln!("<4>the cleanup was not recorded ({error})"))
            .ok();
        let freed: u64 = results.iter().map(|result| result.bytes).sum();
        let detail = format!("{} liberati", size_text(freed));
        self.launcher
            .update_task(
                TASK,
                &json!({
                    "state": if failed == 0 { "done" } else { "failed" },
                    "progress": 100,
                    "detail": detail,
                }),
            )
            .await;
        if settings.notify {
            let (title, tone) = if failed == 0 {
                ("Pulizia completata", "success")
            } else {
                ("Pulizia completata con errori", "warning")
            };
            let message = if failed == 0 {
                format!("{name}, {detail}.")
            } else {
                format!("{name}, {detail}. {failed} voci non pulite.")
            };
            self.launcher.notify(title, &message, tone).await;
        }
        self.set_progress(|progress| *progress = Progress::default());
        recorded
    }
}

fn failed(item: &str, error: String) -> ItemResult {
    ItemResult {
        id: item.to_owned(),
        error: Some(error),
        ..ItemResult::default()
    }
}

/// Parses a launcher result and tags it with the item id.
fn outcome(item: &str, value: Value) -> Result<ItemResult, String> {
    let mut result: ItemResult =
        serde_json::from_value(value).map_err(|error| error.to_string())?;
    result.id = item.to_owned();
    Ok(result)
}

#[cfg(test)]
#[path = "../tests/unit/axum/cleaner.rs"]
mod tests;
