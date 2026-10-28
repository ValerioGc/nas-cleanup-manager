use super::*;
use crate::{
    schedule::{Frequency, RuleSchedule},
    test_support::{database, fake_launcher, FakeLauncher},
};
use axum::body::to_bytes;
use std::{fs, sync::atomic::AtomicU64};

fn temp_dir() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let path = env::temp_dir().join(format!(
        "nas-cleanup-manager-main-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

async fn state() -> (AppState, FakeLauncher) {
    let fake = fake_launcher().await;
    let cleaner = Arc::new(Cleaner::new(database().await, fake.launcher()));
    (
        AppState {
            ui_dir: temp_dir(),
            cleaner,
        },
        fake,
    )
}

fn items(list: &[&str]) -> Json<Items> {
    Json(Items {
        items: list.iter().map(|item| (*item).to_owned()).collect(),
    })
}

fn rule(name: &str, items: &[&str]) -> Json<Rule> {
    Json(Rule {
        id: String::new(),
        name: name.to_owned(),
        enabled: true,
        schedule: RuleSchedule {
            frequency: Frequency::Daily,
            time: "04:00".to_owned(),
            weekday: None,
            day: None,
            month: None,
        },
        items: items.iter().map(|item| (*item).to_owned()).collect(),
    })
}

#[tokio::test]
async fn health_answers_with_the_version() {
    let response = health().await.into_response();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["status"], "ok");
    assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));
}

#[test]
fn api_errors_answer_with_their_status_and_message() {
    for (error, status) in [
        (ApiError::bad_request("no"), StatusCode::BAD_REQUEST),
        (ApiError::not_found("no"), StatusCode::NOT_FOUND),
        (ApiError::conflict("no"), StatusCode::CONFLICT),
        (ApiError::unavailable("no"), StatusCode::SERVICE_UNAVAILABLE),
        (
            ApiError::internal("disk"),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    ] {
        assert_eq!(error.into_response().status(), status);
    }
}

#[tokio::test]
async fn the_page_reads_the_items_measures_them_and_starts_a_cleanup() {
    let (state, fake) = state().await;
    let listed = catalog_entries(State(state.clone())).await;
    assert_eq!(listed.0.len(), catalog::SYSTEM.len() + 2);
    let summary = overview(State(state.clone())).await.expect("overview");
    assert_eq!(summary.0["total"], 1000);

    let measured = preview(State(state.clone()), items(&["temporary"]))
        .await
        .expect("preview");
    assert_eq!(measured.0[0].bytes, 100);
    let refused = preview(State(state.clone()), items(&["/etc"])).await;
    assert_eq!(
        refused.expect_err("unknown").status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        clean(State(state.clone()), items(&[]))
            .await
            .expect_err("empty")
            .status,
        StatusCode::BAD_REQUEST
    );

    assert_eq!(
        clean(State(state.clone()), items(&["temporary"]))
            .await
            .expect("started"),
        StatusCode::ACCEPTED
    );
    assert!(progress(State(state.clone())).await.0.running);
    assert_eq!(
        clean(State(state.clone()), items(&["temporary"]))
            .await
            .expect_err("busy")
            .status,
        StatusCode::CONFLICT
    );
    for _ in 0..100 {
        if !state.cleaner.progress().running {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(
        history(State(state.clone()))
            .await
            .expect("history")
            .0
            .len(),
        1
    );
    assert!(fake.asked().contains(&"POST /api/tasks".to_owned()));

    let alone = AppState {
        cleaner: Arc::new(Cleaner::new(
            database().await,
            launcher::Launcher::new(None),
        )),
        ..state
    };
    assert_eq!(
        overview(State(alone))
            .await
            .expect_err("unavailable")
            .status,
        StatusCode::SERVICE_UNAVAILABLE
    );
}

#[tokio::test]
async fn the_settings_and_the_choice_are_checked_and_kept() {
    let (state, _) = state().await;
    let saved = save_settings(
        State(state.clone()),
        Json(store::Settings {
            temporary_days: 14,
            journal_mb: 200,
            notify: false,
        }),
    )
    .await
    .expect("saved");
    assert_eq!(
        settings(State(state.clone())).await.expect("read").0,
        saved.0
    );
    let wrong = save_settings(
        State(state.clone()),
        Json(store::Settings {
            temporary_days: 0,
            ..saved.0
        }),
    )
    .await;
    assert_eq!(wrong.expect_err("invalid").status, StatusCode::BAD_REQUEST);

    let chosen = save_selection(State(state.clone()), items(&["journal"]))
        .await
        .expect("chosen");
    assert_eq!(chosen.0.items, ["journal"]);
    assert_eq!(
        selection(State(state.clone())).await.expect("kept").0.items,
        ["journal"]
    );
    assert!(save_selection(State(state), items(&["journal", "journal"]))
        .await
        .is_err());
}

#[tokio::test]
async fn scheduled_cleanups_are_created_changed_run_and_removed() {
    let (state, fake) = state().await;
    let created = create_rule(State(state.clone()), rule("Ogni notte", &["temporary"]))
        .await
        .expect("created")
        .0;
    assert!(created.rule.id.starts_with("rule-"));
    assert_eq!(
        create_rule(State(state.clone()), rule("", &["temporary"]))
            .await
            .expect_err("no name")
            .status,
        StatusCode::BAD_REQUEST
    );
    let id = created.rule.id.clone();
    let changed = update_rule(
        State(state.clone()),
        Param(id.clone()),
        rule("Notte", &["journal", "temporary"]),
    )
    .await
    .expect("changed");
    assert_eq!(changed.0.rule.items, ["journal", "temporary"]);
    assert_eq!(
        update_rule(
            State(state.clone()),
            Param("missing".into()),
            rule("x", &["journal"])
        )
        .await
        .expect_err("missing")
        .status,
        StatusCode::NOT_FOUND
    );
    let listed = rules(State(state.clone())).await.expect("rules").0;
    assert_eq!(listed[0].rule.name, "Notte");
    assert_eq!(
        fake.bodies("PUT /api/apps/self/background")
            .last()
            .expect("declared")["needed"],
        true
    );

    assert_eq!(
        run_rule(State(state.clone()), Param(id.clone()))
            .await
            .expect("run"),
        StatusCode::ACCEPTED
    );
    assert_eq!(
        run_rule(State(state.clone()), Param(id.clone()))
            .await
            .expect_err("busy")
            .status,
        StatusCode::CONFLICT
    );
    assert_eq!(
        run_rule(State(state.clone()), Param("missing".into()))
            .await
            .expect_err("missing")
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        delete_rule(State(state.clone()), Param(id.clone()))
            .await
            .expect("removed"),
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        delete_rule(State(state), Param(id))
            .await
            .expect_err("gone")
            .status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn serves_the_interface_and_nothing_outside_it() {
    let (mut state, _) = state().await;
    fs::write(state.ui_dir.join("index.html"), "home").unwrap();
    fs::create_dir_all(state.ui_dir.join("assets")).unwrap();
    fs::write(state.ui_dir.join("assets/app.js"), "script").unwrap();
    state.ui_dir = state.ui_dir.clone();
    for (uri, status, cache) in [
        ("/", StatusCode::OK, Some("no-cache")),
        (
            "/assets/app.js",
            StatusCode::OK,
            Some("public, max-age=31536000, immutable"),
        ),
        ("/api/unknown", StatusCode::NOT_FOUND, None),
        ("/missing.css", StatusCode::NOT_FOUND, None),
        ("/../secret", StatusCode::BAD_REQUEST, None),
    ] {
        let response = static_file(State(state.clone()), uri.parse().unwrap()).await;
        assert_eq!(response.status(), status, "{uri}");
        assert_eq!(
            response
                .headers()
                .get(header::CACHE_CONTROL)
                .and_then(|value| value.to_str().ok()),
            cache,
            "{uri}"
        );
    }
}

#[test]
fn finds_its_database_socket_and_interface() {
    assert_eq!(
        database_path(Some("/tmp/a.sqlite3".into()), None).unwrap(),
        PathBuf::from("/tmp/a.sqlite3")
    );
    assert_eq!(
        database_path(None, Some("/var/lib/a:/var/lib/b".into())).unwrap(),
        PathBuf::from("/var/lib/a/cleanup-manager.sqlite3")
    );
    assert!(database_path(None, None)
        .unwrap()
        .ends_with("cleanup-manager.sqlite3"));
    assert_eq!(
        socket_path(vec!["--listen".into(), "/tmp/a.sock".into()], None).unwrap(),
        PathBuf::from("/tmp/a.sock")
    );
    assert_eq!(
        socket_path(Vec::new(), Some("/run/b.sock".into())).unwrap(),
        PathBuf::from("/run/b.sock")
    );
    assert!(socket_path(vec!["--listen".into()], None).is_err());
    assert!(socket_path(vec!["--other".into()], None).is_err());
    assert!(socket_path(Vec::new(), None).is_err());
    assert!(socket_path(Vec::new(), Some("".into())).is_err());
    assert_eq!(
        ui_dir_for(Some("/ui".into()), Err("unused".into())).unwrap(),
        PathBuf::from("/ui")
    );
    assert!(ui_dir_for(None, Ok("/usr/bin/app".into())).is_ok());
    assert!(ui_dir_for(None, Err("no executable".into())).is_err());
}

#[tokio::test]
async fn serves_on_its_socket_until_it_is_stopped() {
    let dir = temp_dir();
    fs::write(dir.join("index.html"), "home").unwrap();
    let socket = dir.join("app.sock");
    fs::write(&socket, "occupied").unwrap();
    let occupied = run(
        vec!["--listen".into(), socket.clone().into()],
        None,
        Some(dir.clone()),
        dir.join("db.sqlite3"),
        async {},
    )
    .await;
    assert!(occupied.is_err());
    fs::remove_file(&socket).unwrap();
    let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
    let serving = tokio::spawn(run(
        vec!["--listen".into(), socket.clone().into()],
        None,
        Some(dir.clone()),
        dir.join("db.sqlite3"),
        async move {
            let _ = stopped.await;
        },
    ));
    let client = launcher::Launcher::new(Some(socket.clone()));
    let mut settings = None;
    for _ in 0..100 {
        if let Ok(answer) = client
            .request("GET", "/api/settings", None, Duration::from_secs(5))
            .await
        {
            settings = Some(answer);
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(settings.expect("answer").status, 200);
    stop.send(()).unwrap();
    serving.await.unwrap().expect("stopped");
    assert!(!socket.exists());
    assert!(run(
        Vec::new(),
        Some("/tmp/x.sock".into()),
        Some(temp_dir()),
        dir.join("db"),
        async {}
    )
    .await
    .is_err());
}
