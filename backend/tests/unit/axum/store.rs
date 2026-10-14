use super::*;
use crate::{schedule::Frequency, test_support::database};

fn sample(id: &str, enabled: bool) -> Rule {
    Rule {
        id: id.to_owned(),
        name: "Ogni domenica".to_owned(),
        enabled,
        schedule: RuleSchedule {
            frequency: Frequency::Weekly,
            time: "03:00".to_owned(),
            weekday: Some(7),
            day: None,
            month: None,
        },
        items: vec!["temporary".to_owned(), "journal".to_owned()],
    }
}

#[tokio::test]
async fn the_settings_start_from_their_defaults_and_keep_what_is_saved() {
    let pool = database().await;
    assert_eq!(
        settings(&pool).await.expect("settings"),
        Settings::default()
    );
    let changed = Settings {
        temporary_days: 3,
        journal_mb: 64,
        notify: false,
    };
    changed.validate().expect("valid");
    save_settings(&pool, changed).await.expect("saved");
    assert_eq!(settings(&pool).await.expect("settings"), changed);
    for wrong in [
        Settings {
            temporary_days: 0,
            ..changed
        },
        Settings {
            journal_mb: 10,
            ..changed
        },
    ] {
        assert!(wrong.validate().is_err());
    }
}

#[tokio::test]
async fn the_choice_is_kept_for_the_next_time() {
    let pool = database().await;
    assert_eq!(selection(&pool).await.expect("first"), catalog::defaults());
    let chosen = vec!["old-kernels".to_owned()];
    save_selection(&pool, &chosen).await.expect("saved");
    assert_eq!(selection(&pool).await.expect("kept"), chosen);
}

#[tokio::test]
async fn a_scheduled_cleanup_is_created_changed_and_removed() {
    let pool = database().await;
    let first = sample("a", true);
    first.validate().expect("valid");
    assert!(!save_rule(&pool, &first, 1).await.expect("created"));
    assert!(!save_rule(&pool, &sample("b", false), 2)
        .await
        .expect("second"));
    let changed = Rule {
        name: "Settimanale".to_owned(),
        ..first.clone()
    };
    assert!(save_rule(&pool, &changed, 3).await.expect("changed"));
    let listed = rules(&pool).await.expect("rules");
    assert_eq!(
        listed
            .iter()
            .map(|rule| rule.id.as_str())
            .collect::<Vec<_>>(),
        ["a", "b"]
    );
    assert_eq!(listed[0].name, "Settimanale");
    assert_eq!(rule_of_id(&pool, "a").await, Some(changed));
    assert!(delete_rule(&pool, "a").await.expect("deleted"));
    assert!(!delete_rule(&pool, "a").await.expect("gone"));
    assert_eq!(rule_of_id(&pool, "a").await, None);
}

async fn rule_of_id(pool: &SqlitePool, id: &str) -> Option<Rule> {
    rule(pool, id).await.expect("rule")
}

#[test]
fn a_scheduled_cleanup_has_a_name_items_and_a_real_time() {
    for wrong in [
        Rule {
            name: " ".to_owned(),
            ..sample("a", true)
        },
        Rule {
            items: Vec::new(),
            ..sample("a", true)
        },
        Rule {
            items: vec!["/etc".to_owned()],
            ..sample("a", true)
        },
        Rule {
            schedule: RuleSchedule {
                time: "25:00".to_owned(),
                ..sample("a", true).schedule
            },
            ..sample("a", true)
        },
    ] {
        assert!(wrong.validate().is_err(), "{wrong:?}");
    }
}

#[tokio::test]
async fn the_history_keeps_what_went_the_last_first() {
    let pool = database().await;
    let results = [
        ItemResult {
            id: "journal".to_owned(),
            bytes: 100,
            entries: vec!["x".to_owned()],
            more: 3,
            error: None,
        },
        ItemResult {
            id: "temporary".to_owned(),
            error: Some("failed".to_owned()),
            ..ItemResult::default()
        },
    ];
    let first = record(&pool, 10, None, &results).await.expect("recorded");
    assert_eq!(first.freed, 100);
    assert!(first.results[0].entries.is_empty());
    for at in 11..=(10 + HISTORY) {
        record(&pool, at, Some("Ogni domenica"), &results)
            .await
            .expect("recorded");
    }
    let kept = history(&pool).await.expect("history");
    assert_eq!(kept.len(), HISTORY as usize);
    assert_eq!(kept[0].at, 10 + HISTORY);
    assert_eq!(kept[0].rule.as_deref(), Some("Ogni domenica"));
    assert_eq!(kept[0].results[1].error.as_deref(), Some("failed"));
}
