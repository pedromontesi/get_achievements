//! Cliente da API em Rust (a que roda em 127.0.0.1:3001).
//!
//! Uso, em qualquer lugar do front:
//!     let perfil = api::profile().await?;
//!     let jogos  = api::recent_games(Some(5)).await?;
//!
//! Cada função devolve `Result<_, ApiError>`; `ApiError.message` já é um texto
//! pronto pra mostrar na tela.

// A API inteira fica disponível, mesmo as partes que o app ainda não usa.
#![allow(dead_code)]

use gloo_net::http::Request;
use serde::{de::DeserializeOwned, Deserialize};

// Requisições

/// GET /api/health
pub async fn health() -> Result<Health, ApiError> {
    get("/api/health").await
}

/// GET /api/profile
pub async fn profile() -> Result<Profile, ApiError> {
    get("/api/profile").await
}

/// GET /api/games/recent?count=N
pub async fn recent_games(count: Option<u32>) -> Result<Vec<Game>, ApiError> {
    get(&with_query("/api/games/recent", &[("count", count)])).await
}

pub async fn recent_achievements(
    days: Option<u32>,
    limit: Option<u32>,
) -> Result<Vec<Achievement>, ApiError> {
    get(&with_query(
        "/api/achievements/recent",
        &[("days", days), ("limit", limit)],
    ))
    .await
}

pub async fn dashboard() -> Result<Dashboard, ApiError> {
    get("/api/dashboard").await
}

#[derive(Clone, Debug, Deserialize)]
pub struct Health {
    pub status: String,
    pub user: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub user: String,
    pub ulid: Option<String>,
    pub avatar_url: Option<String>,
    pub profile_url: String,
    pub member_since: Option<String>,
    pub motto: Option<String>,
    pub last_activity: Option<String>,
    pub total_points: i64,
    pub total_softcore_points: i64,
    pub total_true_points: i64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub game_id: i64,
    pub title: String,
    pub console_name: Option<String>,
    pub icon_url: Option<String>,
    pub box_art_url: Option<String>,
    pub game_url: String,
    pub last_played: Option<String>,
    pub achievements_total: i64,
    pub achievements_earned: i64,
    pub achievements_earned_hardcore: i64,
    pub points_total: i64,
    pub points_earned: i64,
    pub completion_percent: u8,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Achievement {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub points: i64,
    pub true_ratio: i64,
    pub hardcore: bool,
    pub unlocked_at: Option<String>,
    pub badge_url: Option<String>,
    pub achievement_url: String,
    pub game_id: i64,
    pub game_title: Option<String>,
    pub game_icon_url: Option<String>,
    pub console_name: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dashboard {
    pub profile: Profile,
    pub recent_games: Vec<Game>,
    pub recent_achievements: Vec<Achievement>,
}

#[derive(Clone, Debug)]
pub struct ApiError {
    pub message: String,
}

const API_OFFLINE: &str =
    "Não foi possível falar com a API. Ela está rodando? Inicie com \"cargo run\" na raiz do projeto.";

#[derive(Deserialize)]
struct ErrorBody {
    error: String,
}

async fn get<T: DeserializeOwned>(path: &str) -> Result<T, ApiError> {
    let response = Request::get(path).send().await.map_err(|_| ApiError {
        message: API_OFFLINE.to_string(),
    })?;

    if !response.ok() {
        // Sem corpo JSON (ex.: o proxy do Trunk sem a API do outro lado): API fora do ar.
        let message = response
            .json::<ErrorBody>()
            .await
            .map(|body| body.error)
            .unwrap_or_else(|_| API_OFFLINE.to_string());
        return Err(ApiError { message });
    }

    response.json::<T>().await.map_err(|e| ApiError {
        message: format!("A API respondeu num formato inesperado: {e}"),
    })
}

fn with_query(path: &str, params: &[(&str, Option<u32>)]) -> String {
    let query: Vec<String> = params
        .iter()
        .filter_map(|(name, value)| value.map(|v| format!("{name}={v}")))
        .collect();
    if query.is_empty() {
        path.to_string()
    } else {
        format!("{path}?{}", query.join("&"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn le_o_json_do_dashboard() {
        let json = include_str!("../fixtures/dashboard.json");
        let data: Dashboard = serde_json::from_str(json).expect("tipos não batem com a API");

        assert_eq!(data.profile.user, "MaxMilyin");
        assert_eq!(data.recent_games.len(), 3);
        assert_eq!(data.recent_games[0].completion_percent, 31);
        assert_eq!(data.recent_achievements.len(), 2);
        assert!(data.recent_achievements.iter().any(|a| a.description.is_none()));
    }

    #[test]
    fn le_o_formato_de_erro() {
        let body: ErrorBody =
            serde_json::from_str(r#"{"code":"invalid_key","error":"Chave recusada."}"#).unwrap();
        assert_eq!(body.error, "Chave recusada.");
    }

    #[test]
    fn monta_a_query_string() {
        assert_eq!(with_query("/api/x", &[("a", None), ("b", None)]), "/api/x");
        assert_eq!(
            with_query("/api/x", &[("days", Some(7)), ("limit", None)]),
            "/api/x?days=7"
        );
        assert_eq!(
            with_query("/api/x", &[("days", Some(7)), ("limit", Some(5))]),
            "/api/x?days=7&limit=5"
        );
    }
}
