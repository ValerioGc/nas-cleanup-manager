use super::*;
use crate::test_support::{database, fake_launcher};

#[test]
fn says_a_size_as_a_person_reads_it() {
    assert_eq!(size_text(12), "12 B");
    assert_eq!(size_text(1536), "1.5 KB");
    assert_eq!(size_text(3 * 1024 * 1024 * 1024), "3.0 GB");
}

#[tokio::test]
async fn previews_each_item_through_the_launcher_or_its_cache() {
    let fake = fake_launcher().await;
    let cleaner = Cleaner::new(database().await, fake.launcher());
    let results = cleaner
        .preview(&[
            "rotated-logs".to_owned(),
            "journal".to_owned(),
            "cache:nas-file-manager:previews".to_owned(),
        ])
        .await
        .expect("preview");
    assert_eq!(results[0].id, "rotated-logs");
    assert_eq!(results[0].bytes, 100);
    assert_eq!(results[0].entries, ["/var/log/syslog.1"]);
    assert_eq!(results[1].error.as_deref(), Some("journalctl failed"));
    assert_eq!(results[2].bytes, 1000);
    assert_eq!(
        fake.bodies("POST /api/apps/self/cleanup")[0],
        json!({
            "action": "preview",
            "item": "rotated-logs",
            "options": { "temporaryDays": 7, "journalMb": 500 },
        })
    );
}

#[tokio::test]
async fn the_overview_adds_the_caches_and_the_last_cleanup() {
    let fake = fake_launcher().await;
    let cleaner = Cleaner::new(database().await, fake.launcher());
    let overview = cleaner.overview().await.expect("overview");
    assert_eq!(
        overview["categories"][1],
        json!({ "id": "caches", "bytes": 1000 })
    );
    assert_eq!(overview["last"], Value::Null);
    cleaner.run(vec!["temporary".to_owned()], None).await;
    let overview = cleaner.overview().await.expect("overview");
    assert_eq!(overview["last"]["freed"], 70);
    let alone = Cleaner::new(database().await, Launcher::new(None));
    assert!(alone.overview().await.is_err());
}

#[tokio::test]
async fn a_cleanup_shows_its_progress_records_what_it_freed_and_says_when_it_ends() {
    let fake = fake_launcher().await;
    let cleaner = Arc::new(Cleaner::new(database().await, fake.launcher()));
    let recorded = cleaner
        .run(
            vec![
                "temporary".to_owned(),
                "journal".to_owned(),
                "cache:nas-file-manager:previews".to_owned(),
            ],
            Some("Ogni domenica".to_owned()),
        )
        .await
        .expect("recorded");
    assert_eq!(recorded.freed, 70 + 1000);
    assert_eq!(recorded.rule.as_deref(), Some("Ogni domenica"));
    assert_eq!(
        recorded.results[1].error.as_deref(),
        Some("journalctl failed")
    );
    let asked = fake.asked();
    assert!(asked.contains(&"DELETE /api/caches/nas-file-manager/previews".to_owned()));
    assert_eq!(
        fake.bodies("POST /api/tasks")[0]["name"],
        "Pulizia: Ogni domenica"
    );
    let last = fake.bodies("PATCH /api/tasks/nas-cleanup-manager-run");
    assert_eq!(last.last().expect("end")["state"], "failed");
    let notified = fake.bodies("POST /api/notifications");
    assert_eq!(notified[0]["title"], "Pulizia completata con errori");
    assert!(!cleaner.progress().running);

    // Without the notification of the settings, the end says nothing.
    store::save_settings(
        cleaner.pool(),
        Settings {
            notify: false,
            ..Settings::default()
        },
    )
    .await
    .expect("settings");
    cleaner.run(vec!["temporary".to_owned()], None).await;
    assert_eq!(fake.bodies("POST /api/notifications").len(), 1);
}

#[tokio::test]
async fn one_cleanup_at_a_time() {
    let fake = fake_launcher().await;
    let cleaner = Arc::new(Cleaner::new(database().await, fake.launcher()));
    assert!(cleaner.start(vec!["temporary".to_owned()], None));
    assert!(cleaner.progress().running);
    assert!(!cleaner.start(vec!["temporary".to_owned()], None));
    for _ in 0..100 {
        if !cleaner.progress().running {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert!(!cleaner.progress().running);
    assert_eq!(
        store::history(cleaner.pool()).await.expect("history").len(),
        1
    );
}

#[tokio::test]
async fn keeps_the_next_times_and_works_in_background_only_with_a_rule_on() {
    let fake = fake_launcher().await;
    let cleaner = Cleaner::new(database().await, fake.launcher());
    cleaner.set_next("a", Some(10));
    cleaner.set_next("b", Some(20));
    cleaner.keep_next(&["a".to_owned()]);
    assert_eq!(
        (cleaner.next_run("a"), cleaner.next_run("b")),
        (Some(10), None)
    );
    cleaner.set_next("a", None);
    assert_eq!(cleaner.next_run("a"), None);
    cleaner.set_next("a", Some(10));
    cleaner.forget_next();
    assert_eq!(cleaner.next_run("a"), None);
    cleaner.rules_changed().await;
    assert_eq!(
        fake.bodies("PUT /api/apps/self/background")[0],
        json!({ "needed": false })
    );
}
