use serde::{Deserialize, Deserializer, Serialize};

const MEDIA_HOST: &str = "https://media.retroachievements.org";
const SITE_HOST: &str = "https://retroachievements.org";


#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RawProfile {
    pub user: Option<String>,
    #[serde(rename = "ULID")]
    pub ulid: Option<String>,
    pub user_pic: Option<String>,
    pub member_since: Option<String>,
    pub rich_presence_msg: Option<String>,
    pub motto: Option<String>,
    #[serde(default, deserialize_with = "lenient_i64")]
    pub total_points: i64,
    #[serde(default, deserialize_with = "lenient_i64")]
    pub total_softcore_points: i64,
    #[serde(default, deserialize_with = "lenient_i64")]
    pub total_true_points: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RawGame {
    #[serde(rename = "GameID", default, deserialize_with = "lenient_i64")]
    pub game_id: i64,
    pub title: Option<String>,
    pub console_name: Option<String>,
    pub image_icon: Option<String>,
    pub image_box_art: Option<String>,
    pub last_played: Option<String>,
    #[serde(default, deserialize_with = "lenient_i64")]
    pub num_possible_achievements: i64,
    #[serde(default, deserialize_with = "lenient_i64")]
    pub possible_score: i64,
    #[serde(default, deserialize_with = "lenient_i64")]
    pub num_achieved: i64,
    #[serde(default, deserialize_with = "lenient_i64")]
    pub score_achieved: i64,
    #[serde(default, deserialize_with = "lenient_i64")]
    pub num_achieved_hardcore: i64,
    #[serde(default, deserialize_with = "lenient_i64")]
    pub score_achieved_hardcore: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RawAchievement {
    pub date: Option<String>,
    #[serde(default, deserialize_with = "lenient_bool")]
    pub hardcore_mode: bool,
    #[serde(rename = "AchievementID", default, deserialize_with = "lenient_i64")]
    pub achievement_id: i64,
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(default, deserialize_with = "lenient_string")]
    pub badge_name: Option<String>,
    #[serde(rename = "BadgeURL")]
    pub badge_url: Option<String>,
    #[serde(default, deserialize_with = "lenient_i64")]
    pub points: i64,
    #[serde(default, deserialize_with = "lenient_i64")]
    pub true_ratio: i64,
    pub game_title: Option<String>,
    pub game_icon: Option<String>,
    #[serde(rename = "GameID", default, deserialize_with = "lenient_i64")]
    pub game_id: i64,
    pub console_name: Option<String>,
}

// frontend

#[derive(Serialize)]
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

#[derive(Serialize)]
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

#[derive(Serialize)]
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dashboard {
    pub profile: Profile,
    pub recent_games: Vec<Game>,
    pub recent_achievements: Vec<Achievement>,
}

impl Profile {
    /// `None` quando a RA não devolveu um usuário (usuário inexistente).
    pub fn from_raw(raw: RawProfile) -> Option<Self> {
        let user = raw.user.filter(|u| !u.trim().is_empty())?;
        Some(Self {
            profile_url: format!("{SITE_HOST}/user/{user}"),
            avatar_url: raw.user_pic.as_deref().map(media_url),
            ulid: raw.ulid,
            member_since: raw.member_since,
            motto: raw.motto.filter(|m| !m.trim().is_empty()),
            last_activity: raw.rich_presence_msg.filter(|m| !m.trim().is_empty()),
            total_points: raw.total_points,
            total_softcore_points: raw.total_softcore_points,
            total_true_points: raw.total_true_points,
            user,
        })
    }
}

impl From<RawGame> for Game {
    fn from(raw: RawGame) -> Self {
        let earned = raw.num_achieved.max(raw.num_achieved_hardcore);
        let points = raw.score_achieved.max(raw.score_achieved_hardcore);
        let percent = if raw.num_possible_achievements > 0 {
            (earned * 100 / raw.num_possible_achievements).clamp(0, 100) as u8
        } else {
            0
        };
        Self {
            game_url: format!("{SITE_HOST}/game/{}", raw.game_id),
            game_id: raw.game_id,
            title: raw.title.unwrap_or_else(|| "Jogo sem título".to_string()),
            console_name: raw.console_name,
            icon_url: raw.image_icon.as_deref().map(media_url),
            box_art_url: raw.image_box_art.as_deref().map(media_url),
            last_played: raw.last_played,
            achievements_total: raw.num_possible_achievements,
            achievements_earned: earned,
            achievements_earned_hardcore: raw.num_achieved_hardcore,
            points_total: raw.possible_score,
            points_earned: points,
            completion_percent: percent,
        }
    }
}

impl From<RawAchievement> for Achievement {
    fn from(raw: RawAchievement) -> Self {
        let badge = raw
            .badge_url
            .map(|p| media_url(&p))
            .or_else(|| raw.badge_name.map(|n| format!("{MEDIA_HOST}/Badge/{n}.png")));
        Self {
            achievement_url: format!("{SITE_HOST}/achievement/{}", raw.achievement_id),
            id: raw.achievement_id,
            title: raw.title.unwrap_or_else(|| "Conquista".to_string()),
            description: raw.description.filter(|d| !d.trim().is_empty()),
            points: raw.points,
            true_ratio: raw.true_ratio,
            hardcore: raw.hardcore_mode,
            unlocked_at: raw.date,
            badge_url: badge,
            game_id: raw.game_id,
            game_title: raw.game_title,
            game_icon_url: raw.game_icon.as_deref().map(media_url),
            console_name: raw.console_name,
        }
    }
}

fn media_url(path: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") {
        path.to_string()
    } else {
        format!("{MEDIA_HOST}/{}", path.trim_start_matches('/'))
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Loose {
    Int(i64),
    Float(f64),
    Bool(bool),
    Text(String),
    Null(()),
}

fn lenient_i64<'de, D: Deserializer<'de>>(d: D) -> Result<i64, D::Error> {
    Ok(match Loose::deserialize(d)? {
        Loose::Int(n) => n,
        Loose::Float(f) => f as i64,
        Loose::Bool(b) => b as i64,
        Loose::Text(s) => s.trim().parse().unwrap_or(0),
        Loose::Null(()) => 0,
    })
}

fn lenient_bool<'de, D: Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
    Ok(match Loose::deserialize(d)? {
        Loose::Int(n) => n != 0,
        Loose::Float(f) => f != 0.0,
        Loose::Bool(b) => b,
        Loose::Text(s) => s.trim() == "1" || s.trim().eq_ignore_ascii_case("true"),
        Loose::Null(()) => false,
    })
}

fn lenient_string<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    Ok(match Loose::deserialize(d)? {
        Loose::Int(n) => Some(n.to_string()),
        Loose::Float(f) => Some(f.to_string()),
        Loose::Bool(_) | Loose::Null(()) => None,
        Loose::Text(s) => Some(s).filter(|s| !s.is_empty()),
    })
}
