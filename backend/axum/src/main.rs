mod catalog;
mod cleaner;
mod launcher;
mod schedule;
mod scheduler;
mod store;

use axum::{
    extract::{Path as Param, State},
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use cleaner::Cleaner;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions},
};
use std::{
    env,
    ffi::OsString,
    fmt,
    os::unix::fs::FileTypeExt,
    path::{Component, Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use store::Rule;
use tokio::net::{UnixListener, UnixStream};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[derive(Clone)]
struct AppState {
    ui_dir: PathBuf,
    cleaner: Arc<Cleaner>,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }

    fn conflict(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            message: message.into(),
        }
    }

    fn unavailable(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            message: message.into(),
        }
    }

    fn internal(error: impl fmt::Display) -> Self {
        eprintln!("Cleanup database error: {error}");
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Unable to access the cleanup database".to_owned(),
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        Self::internal(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}

type Answer<T> = Result<Json<T>, ApiError>;

/// Some items, as the page sends them.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Items {
    items: Vec<String>,
}

impl Items {
    fn checked(self) -> Result<Vec<String>, ApiError> {
        catalog::valid(&self.items).map_err(ApiError::bad_request)?;
        Ok(self.items)
    }
}

async fn health() -> impl IntoResponse {
    Json(json!({"status":"ok","version":env!("CARGO_PKG_VERSION")}))
}

async fn catalog_entries(State(state): State<AppState>) -> Json<Vec<catalog::Entry>> {
    Json(catalog::entries(state.cleaner.launcher()).await)
}

async fn overview(State(state): State<AppState>) -> Answer<serde_json::Value> {
    state
        .cleaner
        .overview()
        .await
        .map(Json)
        .map_err(ApiError::unavailable)
}

async fn preview(
    State(state): State<AppState>,
    Json(items): Json<Items>,
) -> Answer<Vec<store::ItemResult>> {
    let items = items.checked()?;
    Ok(Json(state.cleaner.preview(&items).await?))
}

async fn clean(
    State(state): State<AppState>,
    Json(items): Json<Items>,
) -> Result<StatusCode, ApiError> {
    let items = items.checked()?;
    if items.is_empty() {
        return Err(ApiError::bad_request("Choose at least one item."));
    }
    if state.cleaner.start(items, None) {
        Ok(StatusCode::ACCEPTED)
    } else {
        Err(ApiError::conflict("A cleanup is already running."))
    }
}

async fn progress(State(state): State<AppState>) -> Json<cleaner::Progress> {
    Json(state.cleaner.progress())
}

async fn settings(State(state): State<AppState>) -> Answer<store::Settings> {
    Ok(Json(store::settings(state.cleaner.pool()).await?))
}

async fn save_settings(
    State(state): State<AppState>,
    Json(settings): Json<store::Settings>,
) -> Answer<store::Settings> {
    settings.validate().map_err(ApiError::bad_request)?;
    store::save_settings(state.cleaner.pool(), settings).await?;
    Ok(Json(settings))
}

async fn selection(State(state): State<AppState>) -> Answer<Items> {
    Ok(Json(Items {
        items: store::selection(state.cleaner.pool()).await?,
    }))
}

async fn save_selection(State(state): State<AppState>, Json(items): Json<Items>) -> Answer<Items> {
    let items = items.checked()?;
    store::save_selection(state.cleaner.pool(), &items).await?;
    Ok(Json(Items { items }))
}

/// A scheduled cleanup with the time it comes next, for the list.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Listed {
    #[serde(flatten)]
    rule: Rule,
    next: Option<i64>,
}

async fn rules(State(state): State<AppState>) -> Answer<Vec<Listed>> {
    let rules = store::rules(state.cleaner.pool()).await?;
    Ok(Json(
        rules
            .into_iter()
            .map(|rule| Listed {
                next: state.cleaner.next_run(&rule.id),
                rule,
            })
            .collect(),
    ))
}

fn new_id() -> String {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    format!(
        "rule-{}-{}",
        cleaner::now_ms(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}

async fn save(state: &AppState, rule: Rule) -> Answer<Listed> {
    rule.validate().map_err(ApiError::bad_request)?;
    store::save_rule(state.cleaner.pool(), &rule, cleaner::now_ms()).await?;
    scheduler::refresh(&state.cleaner, &rule.id).await;
    state.cleaner.rules_changed().await;
    Ok(Json(Listed {
        next: state.cleaner.next_run(&rule.id),
        rule,
    }))
}

async fn create_rule(State(state): State<AppState>, Json(rule): Json<Rule>) -> Answer<Listed> {
    save(
        &state,
        Rule {
            id: new_id(),
            ..rule
        },
    )
    .await
}

async fn update_rule(
    State(state): State<AppState>,
    Param(id): Param<String>,
    Json(rule): Json<Rule>,
) -> Answer<Listed> {
    if store::rule(state.cleaner.pool(), &id).await?.is_none() {
        return Err(ApiError::not_found("No such scheduled cleanup."));
    }
    save(&state, Rule { id, ..rule }).await
}

async fn delete_rule(
    State(state): State<AppState>,
    Param(id): Param<String>,
) -> Result<StatusCode, ApiError> {
    if !store::delete_rule(state.cleaner.pool(), &id).await? {
        return Err(ApiError::not_found("No such scheduled cleanup."));
    }
    state.cleaner.set_next(&id, None);
    state.cleaner.rules_changed().await;
    Ok(StatusCode::NO_CONTENT)
}

/// Runs a scheduled cleanup now.
async fn run_rule(
    State(state): State<AppState>,
    Param(id): Param<String>,
) -> Result<StatusCode, ApiError> {
    let rule = store::rule(state.cleaner.pool(), &id)
        .await?
        .ok_or_else(|| ApiError::not_found("No such scheduled cleanup."))?;
    if state.cleaner.start(rule.items, Some(rule.name)) {
        Ok(StatusCode::ACCEPTED)
    } else {
        Err(ApiError::conflict("A cleanup is already running."))
    }
}

async fn history(State(state): State<AppState>) -> Answer<Vec<store::Cleanup>> {
    Ok(Json(store::history(state.cleaner.pool()).await?))
}

type Failure = Box<dyn std::error::Error + Send + Sync>;

/// `NAS_DB_PATH` when set, otherwise the database in the first state directory of the unit.
fn database_path(
    configured: Option<OsString>,
    state_directory: Option<OsString>,
) -> Result<PathBuf, String> {
    if let Some(path) = configured.filter(|value| !value.is_empty()) {
        return Ok(PathBuf::from(path));
    }
    if let Some(directory) = state_directory
        .and_then(|value| value.into_string().ok())
        .and_then(|value| value.split(':').next().map(str::to_owned))
        .filter(|value| !value.is_empty())
    {
        return Ok(PathBuf::from(directory).join("cleanup-manager.sqlite3"));
    }
    if cfg!(debug_assertions) {
        return Ok(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../data/cleanup-manager.sqlite3")
        );
    }
    Err("Set NAS_DB_PATH or run under systemd with STATE_DIRECTORY.".to_owned())
}

async fn open_database(path: &Path) -> Result<SqlitePool, Failure> {
    if let Some(directory) = path
        .parent()
        .filter(|directory| !directory.as_os_str().is_empty())
    {
        tokio::fs::create_dir_all(directory).await?;
    }
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;
    MIGRATOR.run(&pool).await?;
    Ok(pool)
}

/// `NAS_UI_DIR` when set, the `dist` of the package beside the binary, or the build of the repository.
fn ui_dir_for(
    configured: Option<PathBuf>,
    executable: Result<PathBuf, String>,
) -> Result<PathBuf, String> {
    if let Some(path) = configured {
        return Ok(path);
    }
    let executable = executable?;
    let binary_dir = executable.parent().ok_or("Invalid executable path")?;
    let packaged = binary_dir
        .parent()
        .ok_or("Invalid executable path")?
        .join("dist");
    if packaged.join("index.html").is_file() {
        Ok(packaged)
    } else {
        Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../dist/axum"))
    }
}

async fn static_file(State(state): State<AppState>, uri: Uri) -> Response {
    let path = uri.path();
    if path.starts_with("/api/") || path == "/api" {
        return StatusCode::NOT_FOUND.into_response();
    }
    let requested = if path == "/" {
        "index.html"
    } else {
        &path[1..]
    };
    let relative = Path::new(requested);
    if !relative
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
    {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let file = state.ui_dir.join(relative);
    let mime = match file
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
    {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        _ => "application/octet-stream",
    };
    // The files of `assets/` carry a hash of their content in the name: the browser keeps them for
    // good. The page that names them is asked for again every time.
    let cache = if requested.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    match tokio::fs::read(file).await {
        Ok(bytes) => (
            [(header::CONTENT_TYPE, mime), (header::CACHE_CONTROL, cache)],
            bytes,
        )
            .into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn prepare_socket(path: &Path) -> Result<(), String> {
    if let Ok(metadata) = tokio::fs::symlink_metadata(path).await {
        if !metadata.file_type().is_socket() {
            return Err(format!("Socket path is occupied: {}", path.display()));
        }
        match UnixStream::connect(path).await {
            Ok(_) => return Err(format!("Socket is already listening: {}", path.display())),
            Err(error) if error.kind() == std::io::ErrorKind::ConnectionRefused => {}
            Err(error) => return Err(format!("Unable to check socket: {error}")),
        }
        tokio::fs::remove_file(path)
            .await
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

/// The socket of `--listen`, or the one the launcher configured.
fn socket_path(args: Vec<OsString>, configured: Option<OsString>) -> Result<PathBuf, String> {
    let mut listen = configured;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        if arg == "--listen" {
            listen = Some(args.next().ok_or("--listen requires a socket path")?);
        } else {
            return Err(format!("Unrecognized argument: {}", arg.to_string_lossy()));
        }
    }
    let path = PathBuf::from(listen.ok_or("Provide --listen or NAS_APP_SOCKET")?);
    if path.as_os_str().is_empty() {
        return Err("Empty socket path".into());
    }
    Ok(path)
}

/// Serves the cleanup and the interface on the socket until `shutdown_signal` completes.
async fn run(
    args: Vec<OsString>,
    configured_socket: Option<OsString>,
    configured_ui: Option<PathBuf>,
    database: PathBuf,
    shutdown_signal: impl std::future::Future<Output = ()> + Send + 'static,
) -> Result<(), Failure> {
    let path = socket_path(args, configured_socket)?;
    let ui_dir = ui_dir_for(
        configured_ui,
        env::current_exe().map_err(|error| error.to_string()),
    )?;
    if !ui_dir.join("index.html").is_file() {
        return Err(format!("Frontend not found: {}", ui_dir.display()).into());
    }
    let database = open_database(&database).await?;
    let cleaner = Arc::new(Cleaner::new(database, launcher::Launcher::from_env()));
    prepare_socket(&path).await?;
    let listener = UnixListener::bind(&path)?;
    let app = router(AppState {
        ui_dir,
        cleaner: Arc::clone(&cleaner),
    });
    println!("nas-cleanup-manager listening on {}", path.display());
    // What runs without the window: the scheduled cleanups, and the launcher told whether any is on.
    let scheduler = tokio::spawn(scheduler::run(Arc::clone(&cleaner)));
    tokio::spawn({
        let cleaner = Arc::clone(&cleaner);
        async move { cleaner.declare_background().await }
    });
    let result = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await;
    scheduler.abort();
    if let Err(error) = tokio::fs::remove_file(&path).await {
        eprintln!("Unable to remove socket: {error}");
    }
    result?;
    Ok(())
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/catalog", get(catalog_entries))
        .route("/api/overview", get(overview))
        .route("/api/preview", post(preview))
        .route("/api/clean", post(clean))
        .route("/api/progress", get(progress))
        .route("/api/settings", get(settings).put(save_settings))
        .route("/api/selection", get(selection).put(save_selection))
        .route("/api/rules", get(rules).post(create_rule))
        .route("/api/rules/{id}", put(update_rule).delete(delete_rule))
        .route("/api/rules/{id}/run", post(run_rule))
        .route("/api/history", get(history))
        .fallback(get(static_file))
        .with_state(state)
}

#[tokio::main]
async fn main() -> Result<(), Failure> {
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    run(
        env::args_os().skip(1).collect(),
        env::var_os("NAS_APP_SOCKET"),
        env::var_os("NAS_UI_DIR").map(PathBuf::from),
        database_path(env::var_os("NAS_DB_PATH"), env::var_os("STATE_DIRECTORY"))?,
        async move {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {},
                _ = terminate.recv() => {},
            }
        },
    )
    .await
}

#[cfg(test)]
#[path = "../../tests/unit/axum/main.rs"]
mod tests;

#[cfg(test)]
#[path = "../../tests/unit/axum/support.rs"]
mod test_support;
