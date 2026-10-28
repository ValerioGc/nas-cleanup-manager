//! Cleanup items. System items mirror the fixed launcher list and every app cache becomes an item
//! with the `cache` prefix. Risky items are never selected by default.

use crate::launcher::Launcher;
use serde::Serialize;

/// System items as (group, id, risky). The ids match the launcher cleanup list.
pub const SYSTEM: [(&str, &str, bool); 10] = [
    ("apt", "apt-archives", false),
    ("apt", "apt-obsolete", false),
    ("apt", "apt-orphans", true),
    ("apt", "apt-configs", true),
    ("kernel", "old-kernels", true),
    ("logs", "rotated-logs", false),
    ("logs", "journal", false),
    ("temporary", "temporary", false),
    ("nas", "nas-backups", false),
    ("nas", "nas-leftovers", false),
];

const CACHE: &str = "cache:";

fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Splits a cache item into its app and cache id.
pub fn cache_of(item: &str) -> Option<(&str, &str)> {
    let (app, id) = item.strip_prefix(CACHE)?.split_once(':')?;
    (identifier(app) && identifier(id)).then_some((app, id))
}

pub fn cache_item(app: &str, id: &str) -> String {
    format!("{CACHE}{app}:{id}")
}

/// Whether the id is a system item or a valid cache item.
pub fn is_item(item: &str) -> bool {
    SYSTEM.iter().any(|(_, id, _)| *id == item) || cache_of(item).is_some()
}

/// Default selection with every item that is not risky.
pub fn defaults() -> Vec<String> {
    SYSTEM
        .iter()
        .filter(|(_, _, risky)| !risky)
        .map(|(_, id, _)| (*id).to_owned())
        .collect()
}

/// Checks that the list holds at most 200 known ids without duplicates.
pub fn valid(items: &[String]) -> Result<(), String> {
    if items.len() > 200 {
        return Err("Too many items.".into());
    }
    for (index, item) in items.iter().enumerate() {
        if !is_item(item) {
            return Err(format!("Unknown item {item}."));
        }
        if items[..index].contains(item) {
            return Err(format!("Item {item} listed twice."));
        }
    }
    Ok(())
}

/// Catalog entry for the page. Caches also carry their label, app and size.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    pub group: String,
    pub risky: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<u64>,
}

/// App cache as listed by the launcher. Trashes have the `trash` kind.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Cache {
    pub app: String,
    pub id: String,
    pub label: String,
    pub bytes: u64,
    #[serde(default)]
    pub kind: Option<String>,
}

impl Cache {
    pub fn is_trash(&self) -> bool {
        self.kind.as_deref() == Some("trash")
    }
}

pub async fn caches(launcher: &Launcher) -> Result<Vec<Cache>, String> {
    let listed = launcher.caches().await?;
    serde_json::from_value(listed).map_err(|error| error.to_string())
}

/// System items followed by the caches and trashes the launcher reports. Without the launcher only
/// the system items are listed.
pub async fn entries(launcher: &Launcher) -> Vec<Entry> {
    let mut entries: Vec<Entry> = SYSTEM
        .iter()
        .map(|(group, id, risky)| Entry {
            id: (*id).to_owned(),
            group: (*group).to_owned(),
            risky: *risky,
            label: None,
            app: None,
            bytes: None,
        })
        .collect();
    for cache in caches(launcher).await.unwrap_or_default() {
        if !identifier(&cache.app) || !identifier(&cache.id) {
            continue;
        }
        let trash = cache.is_trash();
        entries.push(Entry {
            id: cache_item(&cache.app, &cache.id),
            group: if trash { "trash" } else { "caches" }.to_owned(),
            risky: trash,
            label: Some(cache.label),
            app: Some(cache.app),
            bytes: Some(cache.bytes),
        });
    }
    entries
}

#[cfg(test)]
#[path = "../tests/unit/axum/catalog.rs"]
mod tests;
