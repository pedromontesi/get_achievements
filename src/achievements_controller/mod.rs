use std::sync::Arc;

use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    achievements_model::{
        Achievement, Dashboard, Game, Profile, RawAchievement, RawGame, RawProfile,
    },
    error::AppError,
    ra_client::RaClient,
    AppState,
};

type Shared = Arc<AppState>;

const DEFAULT_GAMES: u32 = 8;
const DEFAULT_DAYS: u32 = 30;
const DEFAULT_ACHIEVEMENTS: usize = 20;

pub fn routes() -> Router<Shared> {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/dashboard", get(dashboard))
        .route("/api/profile", get(profile))
        .route("/api/games/recent", get(recent_games))
        .route("/api/achievements/recent", get(recent_achievements))
}

#[derive(Deserialize)]
struct GamesQuery {
    count: Option<u32>,
}

#[derive(Deserialize)]
struct AchievementsQuery {
    days: Option<u32>,
    limit: Option<usize>,
}

async fn health(State(state): State<Shared>) -> Json<Value> {
    Json(json!({ "status": "ok", "user": state.ra.username }))
}

async fn profile(State(state): State<Shared>) -> Result<Json<Profile>, AppError> {
    Ok(Json(fetch_profile(&state.ra).await?))
}

async fn recent_games(
    State(state): State<Shared>,
    Query(q): Query<GamesQuery>,
) -> Result<Json<Vec<Game>>, AppError> {
    Ok(Json(fetch_games(&state.ra, q.count.unwrap_or(DEFAULT_GAMES)).await?))
}

async fn recent_achievements(
    State(state): State<Shared>,
    Query(q): Query<AchievementsQuery>,
) -> Result<Json<Vec<Achievement>>, AppError> {
    let days = q.days.unwrap_or(DEFAULT_DAYS);
    let limit = q.limit.unwrap_or(DEFAULT_ACHIEVEMENTS);
    Ok(Json(fetch_achievements(&state.ra, days, limit).await?))
}

async fn dashboard(State(state): State<Shared>) -> Result<Json<Dashboard>, AppError> {
    let ra = &state.ra;
    let (profile, recent_games, recent_achievements) = tokio::try_join!(
        fetch_profile(ra),
        fetch_games(ra, DEFAULT_GAMES),
        fetch_achievements(ra, DEFAULT_DAYS, DEFAULT_ACHIEVEMENTS),
    )?;
    Ok(Json(Dashboard {
        profile,
        recent_games,
        recent_achievements,
    }))
}

async fn fetch_profile(ra: &RaClient) -> Result<Profile, AppError> {
    let raw: RawProfile = ra.get("API_GetUserProfile", &[]).await?;
    Profile::from_raw(raw).ok_or(AppError::UserNotFound)
}

async fn fetch_games(ra: &RaClient, count: u32) -> Result<Vec<Game>, AppError> {
    let count = count.clamp(1, 50);
    let raw: Vec<RawGame> = ra
        .get("API_GetUserRecentlyPlayedGames", &[("c", count.to_string())])
        .await?;
    Ok(raw.into_iter().map(Game::from).collect())
}

async fn fetch_achievements(
    ra: &RaClient,
    days: u32,
    limit: usize,
) -> Result<Vec<Achievement>, AppError> {
    // A RA recebe a janela em minutos (padrão deles: só a última hora).
    let minutes = days.clamp(1, 365) * 24 * 60;
    let mut raw: Vec<RawAchievement> = ra
        .get("API_GetUserRecentAchievements", &[("m", minutes.to_string())])
        .await?;
    // "AAAA-MM-DD HH:MM:SS" ordena corretamente como texto: mais recentes primeiro.
    raw.sort_by(|a, b| b.date.cmp(&a.date));
    raw.truncate(limit.clamp(1, 100));
    Ok(raw.into_iter().map(Achievement::from).collect())
}
