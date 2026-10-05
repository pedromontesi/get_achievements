use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub enum AppError {
    /// A RetroAchievements respondeu com um status de erro.
    Upstream(u16),
    /// Não foi possível falar com a RetroAchievements (rede, timeout).
    Network,
    /// A resposta veio num formato inesperado.
    BadData,
    /// O usuário configurado não existe.
    UserNotFound,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AppError::Upstream(401) | AppError::Upstream(403) => (
                StatusCode::BAD_GATEWAY,
                "invalid_key",
                "A RetroAchievements recusou a chave. Confira API_KEY no .env e reinicie a API.",
            ),
            AppError::Upstream(404) | AppError::UserNotFound => (
                StatusCode::NOT_FOUND,
                "user_not_found",
                "Usuário não encontrado. Confira RA_USERNAME no .env e reinicie a API.",
            ),
            AppError::Upstream(429) => (
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
                "Muitas requisições à RetroAchievements. Aguarde um minuto e atualize.",
            ),
            AppError::Upstream(_) => (
                StatusCode::BAD_GATEWAY,
                "upstream_error",
                "A RetroAchievements devolveu um erro. Tente novamente em instantes.",
            ),
            AppError::Network => (
                StatusCode::BAD_GATEWAY,
                "network_error",
                "Não foi possível conectar à RetroAchievements. Verifique sua internet e tente de novo.",
            ),
            AppError::BadData => (
                StatusCode::BAD_GATEWAY,
                "bad_data",
                "A RetroAchievements respondeu num formato inesperado.",
            ),
        };
        (status, Json(json!({ "code": code, "error": message }))).into_response()
    }
}
