use axum::{Json, Router, extract::State as AxState, http::StatusCode, routing::get};
use chrono::NaiveDateTime;
use deadpool_postgres::{Config, Pool, Runtime};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use tokio_postgres::NoTls;
use tracing::{error, info, warn};
use uuid::Uuid; // Pour le choix aléatoire

#[derive(Serialize)]
struct HelloResponse {
    message: String,
    language: String,
}

#[derive(Serialize, Deserialize)]
struct Item {
    id: Uuid,
    created_at: NaiveDateTime,
}

#[derive(Clone)]
struct AppState {
    db_pool: Pool,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv_override().ok(); // Load from .env
    tracing_subscriber::fmt::init();
    info!("Démarrage de l'API...");

    // Configuration via URL
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "host=localhost user=user password=password dbname=pings".to_string());
    info!("Database URL [{}]", database_url);
    let mut cfg = Config::new();
    cfg.url = Some(database_url);

    // Création du pool (très simple, pas de vérification immédiate)
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
    let state = AppState { db_pool: pool };

    // Définition des routes/endpoints REST
    let app = Router::new()
        .route("/version", get(|| async { "v2" }))
        .route("/health/live", get(|| async { "Alive" }))
        .route("/health/ready", get(readiness_handler))
        .route("/items", get(get_items).post(create_item))
        .route("/hello", get(hello_handler))
        .with_state(state);

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

// Vérifier que la base de données est disponible
async fn readiness_handler(AxState(state): AxState<AppState>) -> StatusCode {
    // Tente de récupérer une connexion du pool
    match state.db_pool.get().await {
        Ok(client) => {
            // Test de requête simple
            match client.execute("SELECT 1", &[]).await {
                Ok(_) => StatusCode::OK,
                Err(_) => {
                    warn!("BdD indisponible !");
                    StatusCode::SERVICE_UNAVAILABLE
                }
            }
        }
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

// Récupération des objects de la base de données
async fn get_items(AxState(state): AxState<AppState>) -> (StatusCode, Json<Vec<Item>>) {
    let client = match state.db_pool.get().await {
        Ok(c) => c,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    };

    let rows = match client.query("SELECT id, created_at FROM items", &[]).await {
        Ok(r) => r,
        Err(e) => {
            error!("Erreur SQL: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]));
        }
    };

    let items: Vec<Item> = rows
        .iter()
        .map(|row| Item {
            id: row.get(0),
            created_at: row.get(1),
        })
        .collect();

    info!("Envoi de {} items avec leurs timestamps", items.len());
    (StatusCode::OK, Json(items))
}

async fn create_item(AxState(state): AxState<AppState>) -> (StatusCode, Json<Option<Item>>) {
    let client = match state.db_pool.get().await {
        Ok(c) => c,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(None)),
    };

    // On utilise RETURNING pour récupérer les valeurs générées par Postgres
    let row = match client
        .query_one(
            "INSERT INTO items DEFAULT VALUES RETURNING id, created_at",
            &[],
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            error!("Erreur lors de la création : {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(None));
        }
    };

    let new_item = Item {
        id: row.get(0),
        created_at: row.get(1),
    };

    info!("Nouvel item créé avec l'ID : {}", new_item.id);
    (StatusCode::CREATED, Json(Some(new_item)))
}
