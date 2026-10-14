use super::*;
use crate::{
    store::{save_rule, Rule},
    test_support::{database, fake_launcher},
};

fn schedule(
    frequency: Frequency,
    weekday: Option<u8>,
    day: Option<u8>,
    month: Option<u8>,
) -> RuleSchedule {
    RuleSchedule {
        frequency,
        time: "03:30".to_owned(),
        weekday,
        day,
        month,
    }
}

#[test]
fn writes_each_schedule_as_systemd_calendar_expressions() {
    use Frequency::*;
    assert_eq!(
        calendar(&schedule(Daily, None, None, None)),
        ["*-*-* 03:30:00"]
    );
    assert_eq!(
        calendar(&schedule(Weekly, Some(7), None, None)),
        ["Sun *-*-* 03:30:00"]
    );
    assert_eq!(
        calendar(&schedule(Monthly, None, Some(5), None)),
        ["*-*-05 03:30:00"]
    );
    assert_eq!(
        calendar(&schedule(Monthly, None, Some(31), None)),
        ["*-*~01 03:30:00"]
    );
    assert_eq!(
        calendar(&schedule(Monthly, None, Some(30), None)),
        ["*-*-30 03:30:00", "*-02~01 03:30:00"]
    );
    assert_eq!(
        calendar(&schedule(Yearly, None, Some(29), Some(2))),
        ["*-02~01 03:30:00"]
    );
    assert_eq!(
        calendar(&schedule(Yearly, None, Some(4), Some(7))),
        ["*-07-04 03:30:00"]
    );
}

#[test]
fn reads_the_earliest_moment_systemd_answers() {
    let answer = "  Original form: *-*-* 03:30:00\n    (in UTC): Wed 2026-10-14 01:30:00 UTC\n\
                  (in UTC): Tue 2026-10-13 01:30:00 UTC\n";
    assert_eq!(
        parse_elapses(answer),
        utc_milliseconds("2026-10-13", "01:30:00")
    );
    let in_utc = "Normalized form: *-*-* 03:00:00\n    Next elapse: Tue 2026-10-13 03:00:00 UTC\n       From now: 15h left\n";
    assert_eq!(
        parse_elapses(in_utc),
        utc_milliseconds("2026-10-13", "03:00:00")
    );
    let local = "    Next elapse: Tue 2026-10-13 03:00:00 CEST\n";
    assert_eq!(parse_elapses(local), None);
    assert_eq!(parse_elapses("nothing"), None);
    assert_eq!(
        utc_milliseconds("2000-03-01", "00:00:01"),
        Some(951_868_801_000)
    );
    assert_eq!(utc_milliseconds("2026-02-28", "bad"), None);
}

#[tokio::test]
async fn asks_systemd_when_a_schedule_comes_next() {
    let Some(next) = next_elapse(&calendar(&schedule(Frequency::Daily, None, None, None))).await
    else {
        eprintln!("systemd-analyze is not available here");
        return;
    };
    let now = now_ms();
    assert!(next > now && next - now <= 24 * 3600 * 1000 + 3600 * 1000);
    assert_eq!(next_elapse(&["not a calendar".to_owned()]).await, None);
}

#[tokio::test]
async fn starts_a_cleanup_whose_time_just_came_and_skips_one_missed() {
    let fake = fake_launcher().await;
    let cleaner = Arc::new(Cleaner::new(database().await, fake.launcher()));
    let rule = Rule {
        id: "a".to_owned(),
        name: "Ogni giorno".to_owned(),
        enabled: true,
        schedule: schedule(Frequency::Daily, None, None, None),
        items: vec!["temporary".to_owned()],
    };
    save_rule(cleaner.pool(), &rule, 1).await.expect("rule");
    assert_eq!(scheduled(&cleaner).await.len(), 1);
    refresh(&cleaner, "a").await;
    let now = now_ms();
    // Missed long ago, or not yet: nothing starts.
    start_due(
        &cleaner,
        vec![
            (now - LATE_MS - 1, "a".to_owned()),
            (now + 1000, "a".to_owned()),
        ],
        now,
    )
    .await;
    assert!(!cleaner.progress().running);
    start_due(&cleaner, vec![(now - 1000, "a".to_owned())], now).await;
    assert!(cleaner.progress().running);
    assert_eq!(cleaner.progress().rule.as_deref(), Some("Ogni giorno"));
    // A rule gone starts nothing.
    start_due(&cleaner, vec![(now, "missing".to_owned())], now).await;
}
