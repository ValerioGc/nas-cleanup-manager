//! The items a cleanup can take: the fixed list of the launcher, in groups, and one item for each
//! cache an application declares (`cache:<app>:<id>`). An item with a warning is never chosen by
//! default: it is taken only when someone ticks it.

use crate::launcher::Launcher;
use serde::Serialize;

/// The items of the system, by group, with their warning. Their ids are the launcher's.
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

/// The application and the id of a cache item.
pub fn cache_of(item: &str) -> Option<(&str, &str)> {
    let (app, id) = item.strip_prefix(CACHE)?.split_once(':')?;
    (identifier(app) && identifier(id)).then_some((app, id))
}

pub fn cache_item(app: &str, id: &str) -> String {
    format!("{CACHE}{app}:{id}")
}

/// Whether an id names an item: one of the system, or a cache.
pub fn is_item(item: &str) -> bool {
    SYSTEM.iter().any(|(_, id, _)| *id == item) || cache_of(item).is_some()
}

/// What a person finds chosen the first time: every item without a warning.
pub fn defaults() -> Vec<String> {
    SYSTEM
        .iter()
        .filter(|(_, _, risky)| !risky)
        .map(|(_, id, _)| (*id).to_owned())
        .collect()
}

/// A list of items, checked: known ids, each once, at most a few hundred.
pub fn valid(items: &[String]) -> Result<(), String> {
    if items.len() > 200 {
        return Err("Too many items.".into());
    }
    for (index, item) in items.iter().enumerate() {
        if !is_item(item) {
            return Err(format!("Unknown item: {item}"));
        }
        if items[..index].contains(item) {
            return Err(format!("Item listed twice: {item}"));
        }
    }
    Ok(())
}

/// One item as the page lists it: a cache brings its name and size.
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

/// A cache of the NAS, as the launcher lists it: a trash beside them is `trash`.
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
    /// A trash holds what was deleted: emptied, it does not come back.
    pub fn is_trash(&self) -> bool {
        self.kind.as_deref() == Some("trash")
    }
}

pub async fn caches(launcher: &Launcher) -> Result<Vec<Cache>, String> {
    let listed = launcher.caches().await?;
    serde_json::from_value(listed).map_err(|error| error.to_string())
}

/// Every item, the caches and the trashes the launcher lists among them: without the launcher,
/// the system ones.
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
#[path = "../../tests/unit/axum/catalog.rs"]
mod tests;
