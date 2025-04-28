use crate::repositorys::status::StatusRepository;
use chrono::Utc;
use domain::status::{Players, StatusRecord};
use serde::Deserialize;
use shared::error::AppResult;
use sqlx::PgPool;
use std::{fs, time::Duration};
use tokio::time::sleep;

#[derive(Debug, Deserialize)]
pub struct ServerEntry {
    pub id: String,
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub servers: Vec<ServerEntry>,
}

pub async fn start_status_watcher(pool: PgPool) -> AppResult<()> {
    let config_text = fs::read_to_string("config/status.toml")?;
    let config: ServerConfig = toml::from_str(&config_text)?;

    let repo = crate::repositorys::status::PostgresStatusRepository::new(pool);

    tokio::spawn(async move {
        loop {
            for server in &config.servers {
                let result = query_minecraft_status(&server.address, server.port).await;
                let timestamp = Utc::now().timestamp();

                let record = match result {
                    Ok((latency, online_players, max_players)) => StatusRecord {
                        online: true,
                        latency: Some(latency as i32),
                        players: Some(Players {
                            online: online_players as i32,
                            max: max_players as i32,
                        }),
                        timestamp,
                    },
                    Err(_) => StatusRecord {
                        online: false,
                        latency: None,
                        players: None,
                        timestamp,
                    },
                };

                if let Err(err) = repo.insert(&server.id, &record).await {
                    tracing::error!("Failed to insert status for {}: {}", server.id, err);
                }
            }
            sleep(Duration::from_secs(60)).await;
        }
    });

    Ok(())
}

async fn query_minecraft_status(address: &str, port: u16) -> AppResult<(u32, u32, u32)> {
    use mc_query::status;

    let data = status(address, port).await?;

    Ok((0, data.players.online as u32, data.players.max as u32))
}
