use axum::{Json, Router, routing::get};
use rand::seq::SliceRandom;
use serde::Serialize;
use tracing::info;

#[derive(Serialize)]
struct HelloResponse {
    message: String,
    language: String,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv_override().ok(); // Load from .env
    tracing_subscriber::fmt::init();
    info!("Démarrage de l'API...");

    // Définition des routes/endpoints REST
    let app = Router::new()
        .route("/version", get(|| async { "v1" }))
        .route("/health/live", get(|| async { "Alive" }))
        .route("/health/ready", get(|| async { "Ready" }))
        .route("/hello", get(hello_handler));

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Générer un bonjour aléatoire
async fn hello_handler() -> Json<HelloResponse> {
    let greetings = [
        ("Bonjour", "Français"),
        ("Hello", "English"),
        ("Hola", "Español"),
        ("Ciao", "Italiano"),
        ("Guten Tag", "Deutsch"),
        ("Olá", "Português"),
        ("Namaste", "Hindi"),
        ("Konnichiwa", "日本語"),
        ("Salaam", "Persian"),
        ("Ahlan", "Arabic"),
    ];

    // Choix aléatoire dans la liste
    let mut rng = rand::thread_rng();
    let (greet, lang) = greetings.choose(&mut rng).unwrap();

    Json(HelloResponse {
        message: greet.to_string(),
        language: lang.to_string(),
    })
}
