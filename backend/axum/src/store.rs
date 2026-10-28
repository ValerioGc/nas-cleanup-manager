//! Persistence of settings, last selection, scheduled cleanups and history.

use crate::{catalog, schedule::RuleSchedule};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// Maximum number of cleanups kept in the history.
const HISTORY: i64 = 100;
const NAME: usize = 80;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Settings {
    /// Minimum age in days of the temporary files to delete.
    pub temporary_days: u16,
    /// Target journal size in MiB.
    pub journal_mb: u32,
    /// Send a NAS notification when a cleanup ends.
    pub notify: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            temporary_days: 7,
            journal_mb: 500,
            notify: true,
        }
    }
}

impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=3650).contains(&self.temporary_days) {
            return Err("The age of the temporary files is 1 to 3650 days.".into());
        }
        if !(32..=102_400).contains(&self.journal_mb) {
            return Err("The size of the journal is 32 to 102400 MiB.".into());
        }
        Ok(())
    }
}

pub async fn settings(pool: &SqlitePool) -> Result<Settings, sqlx::Error> {
    let row: Option<(i64, i64, i64)> =
        sqlx::query_as("SELECT temporary_days, journal_mb, notify FROM settings WHERE id = 1")
            .fetch_optional(pool)
            .await?;
    Ok(
        row.map_or_else(Settings::default, |(days, journal, notify)| Settings {
            temporary_days: u16::try_from(days).unwrap_or(7),
            journal_mb: u32::try_from(journal).unwrap_or(500),
            notify: notify != 0,
        }),
    )
}

pub async fn save_settings(pool: &SqlitePool, settings: Settings) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT OR REPLACE INTO settings (id, temporary_days, journal_mb, notify) VALUES (1, ?, ?, ?)",
    )
    .bind(i64::from(settings.temporary_days))
    .bind(i64::from(settings.journal_mb))
    .bind(i64::from(settings.notify))
    .execute(pool)
    .await
    .map(drop)
}

/// Last saved selection, or the default items on first use.
pub async fn selection(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
    let saved: Option<String> = sqlx::query_scalar("SELECT items FROM selection WHERE id = 1")
        .fetch_optional(pool)
        .await?;
    Ok(saved
        .and_then(|items| serde_json::from_str(&items).ok())
        .unwrap_or_else(catalog::defaults))
}

pub async fn save_selection(pool: &SqlitePool, items: &[String]) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT OR REPLACE INTO selection (id, items) VALUES (1, ?)")
        .bind(serde_json::to_string(items).unwrap_or_default())
        .execute(pool)
        .await
        .map(drop)
}

/// A scheduled cleanup with its schedule and items.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub schedule: RuleSchedule,
    pub items: Vec<String>,
}

impl Rule {
    pub fn validate(&self) -> Result<(), String> {
        let name = self.name.trim();
        if name.is_empty() || name.chars().count() > NAME || name.chars().any(char::is_control) {
            return Err(format!("The name is 1 to {NAME} characters."));
        }
        if self.items.is_empty() {
            return Err("A scheduled cleanup takes at least one item.".into());
        }
        catalog::valid(&self.items)?;
        self.schedule.validate().map_err(str::to_owned)
    }
}

type RuleRow = (String, String, i64, String, String);

fn rule_of((id, name, enabled, schedule, items): RuleRow) -> Option<Rule> {
    Some(Rule {
        id,
        name,
        enabled: enabled != 0,
        schedule: serde_json::from_str(&schedule).ok()?,
        items: serde_json::from_str(&items).ok()?,
    })
}

pub async fn rules(pool: &SqlitePool) -> Result<Vec<Rule>, sqlx::Error> {
    let rows: Vec<RuleRow> = sqlx::query_as(
        "SELECT id, name, enabled, schedule, items FROM rules ORDER BY created_at, id",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().filter_map(rule_of).collect())
}

pub async fn rule(pool: &SqlitePool, id: &str) -> Result<Option<Rule>, sqlx::Error> {
    let row: Option<RuleRow> =
        sqlx::query_as("SELECT id, name, enabled, schedule, items FROM rules WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?;
    Ok(row.and_then(rule_of))
}

/// Updates a rule or inserts it with `created_at` set to `now`. Returns true when it existed.
pub async fn save_rule(pool: &SqlitePool, rule: &Rule, now: i64) -> Result<bool, sqlx::Error> {
    let schedule = serde_json::to_string(&rule.schedule).unwrap_or_default();
    let items = serde_json::to_string(&rule.items).unwrap_or_default();
    let changed =
        sqlx::query("UPDATE rules SET name = ?, enabled = ?, schedule = ?, items = ? WHERE id = ?")
            .bind(rule.name.trim())
            .bind(i64::from(rule.enabled))
            .bind(&schedule)
            .bind(&items)
            .bind(&rule.id)
            .execute(pool)
            .await?
            .rows_affected();
    if changed > 0 {
        return Ok(true);
    }
    sqlx::query(
        "INSERT INTO rules (id, name, enabled, schedule, items, created_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&rule.id)
    .bind(rule.name.trim())
    .bind(i64::from(rule.enabled))
    .bind(&schedule)
    .bind(&items)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(false)
}

pub async fn delete_rule(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    Ok(sqlx::query("DELETE FROM rules WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected()
        > 0)
}

/// Result of an item in a preview or a cleanup.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ItemResult {
    #[serde(default)]
    pub id: String,
    pub bytes: u64,
    #[serde(default)]
    pub entries: Vec<String>,
    #[serde(default)]
    pub more: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Cleanup {
    pub id: i64,
    pub at: i64,
    pub rule: Option<String>,
    pub freed: u64,
    pub results: Vec<ItemResult>,
}

pub async fn record(
    pool: &SqlitePool,
    at: i64,
    rule: Option<&str>,
    results: &[ItemResult],
) -> Result<Cleanup, sqlx::Error> {
    // Drop the file lists, the history keeps only the totals.
    let kept: Vec<ItemResult> = results
        .iter()
        .map(|result| ItemResult {
            entries: Vec::new(),
            more: 0,
            ..result.clone()
        })
        .collect();
    let freed: u64 = kept.iter().map(|result| result.bytes).sum();
    let id = sqlx::query("INSERT INTO history (at, rule, freed, results) VALUES (?, ?, ?, ?)")
        .bind(at)
        .bind(rule)
        .bind(i64::try_from(freed).unwrap_or(i64::MAX))
        .bind(serde_json::to_string(&kept).unwrap_or_default())
        .execute(pool)
        .await?
        .last_insert_rowid();
    sqlx::query(
        "DELETE FROM history WHERE id NOT IN (SELECT id FROM history ORDER BY id DESC LIMIT ?)",
    )
    .bind(HISTORY)
    .execute(pool)
    .await?;
    Ok(Cleanup {
        id,
        at,
        rule: rule.map(str::to_owned),
        freed,
        results: kept,
    })
}

/// Completed cleanups, most recent first.
pub async fn history(pool: &SqlitePool) -> Result<Vec<Cleanup>, sqlx::Error> {
    let rows: Vec<(i64, i64, Option<String>, i64, String)> =
        sqlx::query_as("SELECT id, at, rule, freed, results FROM history ORDER BY id DESC")
            .fetch_all(pool)
            .await?;
    Ok(rows
        .into_iter()
        .map(|(id, at, rule, freed, results)| Cleanup {
            id,
            at,
            rule,
            freed: u64::try_from(freed).unwrap_or_default(),
            results: serde_json::from_str(&results).unwrap_or_default(),
        })
        .collect())
}

#[cfg(test)]
#[path = "../tests/unit/axum/store.rs"]
mod tests;
