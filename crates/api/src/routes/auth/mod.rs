use std::{
    collections::HashMap,
    env,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    Json,
    extract::Extension,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Default)]
pub struct OAuthStateStore {
    inner: Mutex<HashMap<String, Instant>>,
}

impl OAuthStateStore {
    pub async fn insert(&self, state: String) {
        let mut map = self.inner.lock().await;
        map.insert(state, Instant::now());
    }

    pub async fn consume(&self, state: &str) -> bool {
        let mut map = self.inner.lock().await;
        Self::cleanup_locked(&mut map);
        map.remove(state).is_some()
    }

    fn cleanup_locked(map: &mut HashMap<String, Instant>) {
        let ttl = Duration::from_secs(10 * 60);
        let now = Instant::now();
        map.retain(|_, t| now.duration_since(*t) < ttl);
    }
}

#[derive(Debug, Serialize)]
pub struct DiscordLoginResponse {
    pub url: String,
}

pub async fn discord_login(Extension(store): Extension<Arc<OAuthStateStore>>) -> impl IntoResponse {
    let client_id = match env::var("DISCORD_CLIENT_ID") {
        Ok(v) => v,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": 500,
                    "code": "misconfigured",
                    "message": "DISCORD_CLIENT_ID is not set"
                })),
            )
                .into_response();
        }
    };

    let redirect_uri = match env::var("DISCORD_REDIRECT_URI") {
        Ok(v) => v,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "status": 500,
                    "code": "misconfigured",
                    "message": "DISCORD_REDIRECT_URI is not set"
                })),
            )
                .into_response();
        }
    };

    let state = uuid::Uuid::new_v4().to_string();
    store.insert(state.clone()).await;

    let url = format!(
        "https://discord.com/api/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
        urlencoding::encode(&client_id),
        urlencoding::encode(&redirect_uri),
        urlencoding::encode("identify guilds.members.read"),
        urlencoding::encode(&state),
    );

    Json(DiscordLoginResponse { url }).into_response()
}

#[derive(Debug, Deserialize)]
pub struct DiscordExchangeRequest {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct ActorUser {
    pub id: String,
    pub username: String,
    pub global_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DiscordExchangeResponse {
    pub user: ActorUser,
}

#[derive(Debug, Deserialize)]
struct DiscordTokenResponse {
    access_token: String,
    token_type: String,
}

#[derive(Debug, Deserialize)]
struct DiscordUser {
    id: String,
    username: String,
    global_name: Option<String>,
    avatar: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DiscordGuildMember {
    roles: Vec<String>,
}

pub async fn discord_exchange(
    Extension(store): Extension<Arc<OAuthStateStore>>,
    headers: HeaderMap,
    Json(payload): Json<DiscordExchangeRequest>,
) -> impl IntoResponse {
    if payload.code.trim().is_empty() || payload.state.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": 400,
                "code": "bad_request",
                "message": "code/state is required"
            })),
        )
            .into_response();
    }

    if !store.consume(&payload.state).await {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": 400,
                "code": "invalid_state",
                "message": "state is invalid or expired"
            })),
        )
            .into_response();
    }

    let client_id = env::var("DISCORD_CLIENT_ID").unwrap_or_default();
    let client_secret = env::var("DISCORD_CLIENT_SECRET").unwrap_or_default();
    let redirect_uri = env::var("DISCORD_REDIRECT_URI").unwrap_or_default();
    let guild_id = env::var("DISCORD_GUILD_ID").unwrap_or_default();
    let allowed_roles = env::var("DISCORD_ALLOWED_ROLE_IDS").unwrap_or_default();

    if client_id.is_empty()
        || client_secret.is_empty()
        || redirect_uri.is_empty()
        || guild_id.is_empty()
        || allowed_roles.is_empty()
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": 500,
                "code": "misconfigured",
                "message": "Discord OAuth env vars are not fully set"
            })),
        )
            .into_response();
    }

    let _ = headers;

    let http = reqwest::Client::new();

    let token = match http
        .post("https://discord.com/api/oauth2/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("grant_type", "authorization_code"),
            ("code", payload.code.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
        ])
        .send()
        .await
    {
        Ok(res) => {
            if !res.status().is_success() {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "status": 401,
                        "code": "token_exchange_failed",
                        "message": "failed to exchange code"
                    })),
                )
                    .into_response();
            }
            match res.json::<DiscordTokenResponse>().await {
                Ok(v) => v,
                Err(_) => {
                    return (
                        StatusCode::UNAUTHORIZED,
                        Json(serde_json::json!({
                            "status": 401,
                            "code": "token_exchange_failed",
                            "message": "invalid token response"
                        })),
                    )
                        .into_response();
                }
            }
        }
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "status": 401,
                    "code": "token_exchange_failed",
                    "message": "failed to contact discord"
                })),
            )
                .into_response();
        }
    };

    let auth_header = format!("{} {}", token.token_type, token.access_token);

    let discord_user = match http
        .get("https://discord.com/api/users/@me")
        .header("Authorization", auth_header.clone())
        .send()
        .await
    {
        Ok(res) => {
            if !res.status().is_success() {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "status": 401,
                        "code": "discord_user_failed",
                        "message": "failed to fetch discord user"
                    })),
                )
                    .into_response();
            }
            match res.json::<DiscordUser>().await {
                Ok(v) => v,
                Err(_) => {
                    return (
                        StatusCode::UNAUTHORIZED,
                        Json(serde_json::json!({
                            "status": 401,
                            "code": "discord_user_failed",
                            "message": "invalid discord user response"
                        })),
                    )
                        .into_response();
                }
            }
        }
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "status": 401,
                    "code": "discord_user_failed",
                    "message": "failed to contact discord"
                })),
            )
                .into_response();
        }
    };

    let member_url = format!(
        "https://discord.com/api/users/@me/guilds/{}/member",
        urlencoding::encode(&guild_id)
    );

    let member = match http
        .get(member_url)
        .header("Authorization", auth_header)
        .send()
        .await
    {
        Ok(res) => {
            if res.status() == StatusCode::FORBIDDEN {
                return (
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({
                        "status": 403,
                        "code": "not_a_member",
                        "message": "not a guild member"
                    })),
                )
                    .into_response();
            }
            if !res.status().is_success() {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "status": 401,
                        "code": "guild_member_failed",
                        "message": "failed to fetch guild member"
                    })),
                )
                    .into_response();
            }
            match res.json::<DiscordGuildMember>().await {
                Ok(v) => v,
                Err(_) => {
                    return (
                        StatusCode::UNAUTHORIZED,
                        Json(serde_json::json!({
                            "status": 401,
                            "code": "guild_member_failed",
                            "message": "invalid guild member response"
                        })),
                    )
                        .into_response();
                }
            }
        }
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "status": 401,
                    "code": "guild_member_failed",
                    "message": "failed to contact discord"
                })),
            )
                .into_response();
        }
    };

    let allowed: Vec<&str> = allowed_roles
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let has_allowed_role = member.roles.iter().any(|r| allowed.contains(&r.as_str()));

    if !has_allowed_role {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "status": 403,
                "code": "insufficient_role",
                "message": "missing required role"
            })),
        )
            .into_response();
    }

    let avatar_url = discord_user.avatar.as_ref().map(|hash| {
        format!(
            "https://cdn.discordapp.com/avatars/{}/{}.png?size=128",
            discord_user.id, hash
        )
    });

    Json(DiscordExchangeResponse {
        user: ActorUser {
            id: discord_user.id,
            username: discord_user.username,
            global_name: discord_user.global_name,
            avatar_url,
        },
    })
    .into_response()
}
