use async_trait::async_trait;
use domain::status::{StatusResponse, StatusSummary};
use infrastructure::repositorys::status::StatusRepository;
use shared::error::AppResult;

pub struct StatusUsecaseImpl<R: StatusRepository + Send + Sync + 'static> {
    pub repo: R,
}

impl<R: StatusRepository + Send + Sync + 'static> StatusUsecaseImpl<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

#[async_trait]
pub trait StatusUsecase: Send + Sync {
    async fn find_all(&self) -> AppResult<Vec<StatusSummary>>;
    async fn find_by_id(&self, id: &str) -> AppResult<StatusResponse>;
}

#[async_trait]
impl<R: StatusRepository + Send + Sync + 'static> StatusUsecase for StatusUsecaseImpl<R> {
    async fn find_all(&self) -> AppResult<Vec<StatusSummary>> {
        let records = self.repo.list_latest().await?;

        let mut summaries = Vec::new();

        for (id, record) in records {
            let history = self.repo.get_history(&id).await?;

            summaries.push(StatusSummary {
                id,
                online: record.online,
                latency: record.latency,
                players: record.players,
                timestamp: record.timestamp,
                history,
            });
        }

        Ok(summaries)
    }

    async fn find_by_id(&self, id: &str) -> AppResult<StatusResponse> {
        let latest = self.repo.get_latest(id).await?;
        let history = self.repo.get_history(id).await?;

        match latest {
            Some(record) => Ok(StatusResponse {
                id: id.to_string(),
                online: record.online,
                latency: record.latency,
                players: record.players,
                timestamp: record.timestamp,
                history,
            }),
            None => Err(anyhow::anyhow!("Server not found")),
        }
    }
}
