use super::*;
use crate::test_support::fake_launcher;

#[test]
fn reads_plain_and_chunked_answers() {
    let plain = parse(b"HTTP/1.1 204 No Content\r\ncontent-length: 0\r\n\r\n").expect("plain");
    assert_eq!(
        plain,
        Answer {
            status: 204,
            body: String::new()
        }
    );
    let chunked = parse(
        "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\n{\"a\"\r\n6\r\n:\"è\"}\r\n0\r\n\r\n"
            .as_bytes(),
    )
    .expect("chunked");
    assert_eq!(chunked.body, "{\"a\":\"è\"}");
    assert!(parse(b"not http").is_err());
    assert!(parse(b"HTTP/1.1 nope\r\n\r\n").is_err());
}

#[tokio::test]
async fn speaks_to_the_launcher_and_says_why_it_refused() {
    let fake = fake_launcher().await;
    let launcher = fake.launcher();
    let preview = launcher
        .cleanup(&json!({ "action": "preview", "item": "rotated-logs" }))
        .await
        .expect("preview");
    assert_eq!(preview["bytes"], 100);
    let refused = launcher
        .cleanup(&json!({ "action": "preview", "item": "journal" }))
        .await
        .expect_err("refused");
    assert_eq!(refused, "journalctl failed");
    assert_eq!(
        launcher.caches().await.expect("caches")[0]["id"],
        "previews"
    );
    launcher
        .clear_cache("nas-file-manager", "previews")
        .await
        .expect("cleared");
    launcher.background(true).await;
    launcher.start_task("task", "Pulizia").await;
    launcher
        .update_task("task", &json!({ "progress": 50 }))
        .await;
    launcher
        .notify("Pulizia completata", "Liberati 1 GB", "success")
        .await;
    assert_eq!(
        fake.asked(),
        [
            "POST /api/apps/self/cleanup",
            "POST /api/apps/self/cleanup",
            "GET /api/caches",
            "DELETE /api/caches/nas-file-manager/previews",
            "PUT /api/apps/self/background",
            "POST /api/tasks",
            "PATCH /api/tasks/task",
            "POST /api/notifications",
        ]
    );
    assert_eq!(fake.bodies("POST /api/notifications")[0]["source"], ID);

    let alone = Launcher::new(None);
    alone.background(true).await;
    assert_eq!(
        alone.caches().await.expect_err("no launcher"),
        "the launcher is not reachable"
    );
}
