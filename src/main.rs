mod achievements_controller;
mod achievements_model;
mod config;
mod error;
mod ra_client;

use std::sync::Arc;

use config::Config;
use ra_client::RaClient;

pub struct AppState {
    pub ra: RaClient,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let config = Config::from_env().unwrap_or_else(|msg| {
        eprintln!("Erro de configuração: {msg}");
        std::process::exit(1);
    });

    let addr = format!("0.0.0.0:{}", config.port);
    let state = Arc::new(AppState {
        ra: RaClient::new(&config),
    });
    let app = achievements_controller::routes().with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Não foi possível abrir {addr}: {e}. A porta já está em uso? Troque PORT no .env.");
            std::process::exit(1);
        });

    println!("API pronta em http://{addr} (usuário RetroAchievements: {})", config.username);
    axum::serve(listener, app).await.expect("o servidor parou");
}
