use async_trait::async_trait;
use domain::tickets::Ticket;
use shared::error::AppResult;
use sqlx::{PgPool, Row};

#[async_trait]
pub trait TicketRepository {
    async fn fetch_all(&self, user_id: Option<String>) -> AppResult<Vec<Ticket>>;
    async fn find_by_id(&self, id: &str) -> AppResult<Ticket>;
    async fn insert(&self, ticket: Ticket) -> AppResult<()>;
    async fn update(&self, id: &str, ticket: Ticket) -> AppResult<()>;
    async fn delete(&self, id: &str) -> AppResult<()>;
}

pub struct PostgresTicketRepository {
    pub pool: PgPool,
}

impl PostgresTicketRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TicketRepository for PostgresTicketRepository {
    async fn fetch_all(&self, user_id: Option<String>) -> AppResult<Vec<Ticket>> {
        let rows = if let Some(uid) = user_id {
            sqlx::query("SELECT * FROM tickets WHERE user_id = $1")
                .bind(uid)
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query("SELECT * FROM tickets")
                .fetch_all(&self.pool)
                .await?
        };

        let tickets = rows
            .into_iter()
            .map(|row| Ticket {
                id: row.get("id"),
                user_id: row.get("user_id"),
                title: row.get("title"),
                status: row.get("status"),
                messages: serde_json::from_value(row.get("messages")).unwrap_or_default(),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(tickets)
    }

    async fn find_by_id(&self, id: &str) -> AppResult<Ticket> {
        let row = sqlx::query("SELECT * FROM tickets WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(Ticket {
            id: row.get("id"),
            user_id: row.get("user_id"),
            title: row.get("title"),
            status: row.get("status"),
            messages: serde_json::from_value(row.get("messages")).unwrap_or_default(),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn insert(&self, ticket: Ticket) -> AppResult<()> {
        sqlx::query("INSERT INTO tickets (id, user_id, title, status, messages, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(&ticket.id)
            .bind(&ticket.user_id)
            .bind(&ticket.title)
            .bind(&ticket.status)
            .bind(serde_json::to_value(&ticket.messages)?)
            .bind(ticket.created_at)
            .bind(ticket.updated_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update(&self, id: &str, ticket: Ticket) -> AppResult<()> {
        sqlx::query("UPDATE tickets SET title = $1, status = $2, messages = $3, updated_at = $4 WHERE id = $5")
            .bind(ticket.title)
            .bind(ticket.status)
            .bind(serde_json::to_value(ticket.messages)?)
            .bind(ticket.updated_at)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete(&self, id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM tickets WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
