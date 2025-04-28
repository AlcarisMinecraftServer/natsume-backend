use async_trait::async_trait;
use domain::status::{StatusRecord, Players};
use shared::error::AppResult;
use sqlx::{PgPool, Row};

#[async_trait]
pub trait StatusRepository {
    async fn get_latest(&self, id: &str) -> AppResult<Option<StatusRecord>>;
    async fn get_history(&self, id: &str) -> AppResult<Vec<StatusRecord>>;
    async fn insert(&self, id: &str, record: &StatusRecord) -> AppResult<()>;
    async fn list_latest(&self) -> AppResult<Vec<(String, StatusRecord)>>;
}

pub struct PostgresStatusRepository {
    pub pool: PgPool,
}

impl PostgresStatusRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StatusRepository for PostgresStatusRepository {
    async fn get_latest(&self, id: &str) -> AppResult<Option<StatusRecord>> {
        let row = sqlx::query(
            "SELECT online, latency, players_online, players_max, timestamp FROM status WHERE server_id = $1 ORDER BY timestamp DESC LIMIT 1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| StatusRecord {
            online: row.get("online"),
            latency: row.get("latency"),
            players: row.try_get("players_online").ok().map(|online| Players {
                online,
                max: row.get("players_max"),
            }),
            timestamp: row.get("timestamp"),
        }))
    }

    async fn get_history(&self, id: &str) -> AppResult<Vec<StatusRecord>> {
        let rows = sqlx::query(
            "SELECT online, latency, players_online, players_max, timestamp FROM status WHERE server_id = $1 ORDER BY timestamp DESC OFFSET 1 LIMIT 59"
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| StatusRecord {
            online: row.get("online"),
            latency: row.get("latency"),
            players: row.try_get("players_online").ok().map(|online| Players {
                online,
                max: row.get("players_max"),
            }),
            timestamp: row.get("timestamp"),
        }).collect())
    }

    async fn insert(&self, id: &str, record: &StatusRecord) -> AppResult<()> {
        let (players_online, players_max) = match &record.players {
            Some(p) => (Some(p.online as i32), Some(p.max as i32)),
            None => (None, None),
        };

        sqlx::query(
            "INSERT INTO status (server_id, online, latency, players_online, players_max, timestamp) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(id)
        .bind(record.online)
        .bind(record.latency)
        .bind(players_online)
        .bind(players_max)
        .bind(record.timestamp)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn list_latest(&self) -> AppResult<Vec<(String, StatusRecord)>> {
        let rows = sqlx::query(
            "
            SELECT DISTINCT ON (server_id) server_id, online, latency, players_online, players_max, timestamp
            FROM status
            ORDER BY server_id, timestamp DESC
            "
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| {
            (
                row.get("server_id"),
                StatusRecord {
                    online: row.get("online"),
                    latency: row.get("latency"),
                    players: row.try_get("players_online").ok().map(|online| Players {
                        online,
                        max: row.get("players_max"),
                    }),
                    timestamp: row.get("timestamp"),
                }
            )
        }).collect())
    }
}
