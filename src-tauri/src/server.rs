use axum::{
    extract::Json,
    http::{HeaderValue, Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_http::cors::{AllowHeaders, CorsLayer};

use crate::claude_config;

const PORT: u16 = 47821;
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Verifica si Node.js está instalado y retorna su versión.
fn check_nodejs() -> (bool, String) {
    match std::process::Command::new("node").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            (true, version)
        }
        _ => (false, String::new()),
    }
}

// ── Estructuras de request/response ────────────────────────────────────────

#[derive(Deserialize)]
pub struct ConnectBody {
    pub nombre: String,
    pub url: String,
    pub token: String,
}

#[derive(Deserialize)]
pub struct DisconnectBody {
    pub nombre: String,
}

#[derive(Serialize)]
struct ApiError {
    ok: bool,
    error: String,
    mensaje: String,
}

fn error_response(status: StatusCode, code: &str, mensaje: &str) -> impl IntoResponse {
    (
        status,
        Json(ApiError {
            ok: false,
            error: code.into(),
            mensaje: mensaje.into(),
        }),
    )
}

// ── Dominios permitidos de Lexiius (para validar URLs entrantes) ────────────

const ALLOWED_DOMAINS: &[&str] = &[
    "anonimizador-backend-prod-production.up.railway.app",
    "api.lexiius.com",
    "localhost",
    "127.0.0.1",
];

fn is_valid_lexiius_url(url: &str) -> bool {
    ALLOWED_DOMAINS.iter().any(|d| url.contains(d))
}

// ── Handlers ───────────────────────────────────────────────────────────────

async fn ping_handler() -> impl IntoResponse {
    let platform = if cfg!(target_os = "windows") { "windows" } else { "macos" };
    Json(json!({ "ok": true, "version": VERSION, "platform": platform }))
}

async fn node_check_handler() -> impl IntoResponse {
    let (ok, version) = check_nodejs();
    Json(json!({
        "ok": ok,
        "version": version,
        "mensaje": if ok { "" } else {
            "Node.js no está instalado. Descargalo en https://nodejs.org"
        }
    }))
}

async fn connect_handler(Json(body): Json<ConnectBody>) -> impl IntoResponse {
    // Validaciones
    if body.nombre.trim().is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "nombre_vacio", "El nombre no puede estar vacío").into_response();
    }
    if !body.token.starts_with("anon_live_") {
        return error_response(StatusCode::BAD_REQUEST, "token_invalido", "El token debe comenzar con anon_live_").into_response();
    }
    if !is_valid_lexiius_url(&body.url) {
        return error_response(StatusCode::BAD_REQUEST, "url_no_permitida", "La URL debe apuntar a un dominio de Lexiius").into_response();
    }

    // Verificar Node.js ANTES de escribir el config
    let (node_ok, node_version) = check_nodejs();
    if !node_ok {
        return error_response(
            StatusCode::BAD_REQUEST,
            "nodejs_requerido",
            "Node.js no está instalado. Descargalo gratis en https://nodejs.org — tarda 2 minutos.",
        ).into_response();
    }

    let claude_running = claude_config::is_claude_running();

    match claude_config::write_connection(&body.nombre, &body.url, &body.token) {
        Ok(path) => (
            StatusCode::OK,
            Json(json!({
                "ok": true,
                "claude_running": claude_running,
                "reiniciar_requerido": claude_running,
                "config_path": path.to_string_lossy(),
                "nodejs_version": node_version,
            })),
        ).into_response(),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "escritura_fallida", &e).into_response(),
    }
}

async fn disconnect_handler(Json(body): Json<DisconnectBody>) -> impl IntoResponse {
    if body.nombre.trim().is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "nombre_vacio", "El nombre no puede estar vacío").into_response();
    }

    let claude_running = claude_config::is_claude_running();

    match claude_config::remove_connection(&body.nombre) {
        Ok((path, existia)) => (
            StatusCode::OK,
            Json(json!({
                "ok": true,
                "existia": existia,
                "claude_running": claude_running,
                "reiniciar_requerido": claude_running && existia,
                "config_path": path.to_string_lossy()
            })),
        ).into_response(),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "error_interno", &e).into_response(),
    }
}

async fn connections_handler() -> impl IntoResponse {
    let path = claude_config::get_config_path();
    let config_existe = path.exists();

    match claude_config::list_lexiius_connections() {
        Ok(connections) => (
            StatusCode::OK,
            Json(json!({
                "ok": true,
                "connections": connections,
                "config_path": path.to_string_lossy(),
                "config_existe": config_existe
            })),
        ).into_response(),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "error_interno", &e).into_response(),
    }
}

// ── Inicio del servidor ────────────────────────────────────────────────────

pub async fn start() {
    // CORS con soporte para Private Network Access (requerido por Chrome
    // cuando una página HTTPS accede a localhost HTTP)
    let cors = CorsLayer::new()
        .allow_origin([
            "https://app.lexiius.com".parse::<HeaderValue>().unwrap(),
            "https://lexiius.com".parse::<HeaderValue>().unwrap(),
            "http://localhost:5173".parse::<HeaderValue>().unwrap(),
            "http://localhost:3000".parse::<HeaderValue>().unwrap(),
        ])
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(AllowHeaders::any())
        .allow_private_network(true);

    let app = Router::new()
        .route("/ping", get(ping_handler))
        .route("/node-check", get(node_check_handler))
        .route("/connect", post(connect_handler))
        .route("/disconnect", post(disconnect_handler))
        .route("/connections", get(connections_handler))
        .layer(cors);

    let addr = format!("127.0.0.1:{}", PORT);
    let listener = tokio::net::TcpListener::bind(&addr).await
        .unwrap_or_else(|e| panic!("No se pudo iniciar el servidor en {}: {}", addr, e));

    println!("Lexiius Connector corriendo en http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
