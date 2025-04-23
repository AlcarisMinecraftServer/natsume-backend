use async_trait::async_trait;

use domain::tickets::Ticket;
use infrastructure::repositorys::ticket::TicketRepository;
use shared::error::AppResult;

pub struct TicketUsecaseImpl<R: TicketRepository + Send + Sync> {
    pub repo: R,
}

impl<R: TicketRepository + Send + Sync> TicketUsecaseImpl<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

#[async_trait]
pub trait TicketUsecase: Send + Sync {
    async fn find_all(&self, user_id: Option<String>) -> AppResult<Vec<Ticket>>;
    async fn find_by_id(&self, id: &str) -> AppResult<Ticket>;
    async fn create(&self, ticket: Ticket) -> AppResult<()>;
    async fn update(&self, id: &str, ticket: Ticket) -> AppResult<()>;
    async fn delete(&self, id: &str) -> AppResult<()>;
}

#[async_trait]
impl<R: TicketRepository + Send + Sync> TicketUsecase for TicketUsecaseImpl<R> {
    async fn find_all(&self, user_id: Option<String>) -> AppResult<Vec<Ticket>> {
        self.repo.fetch_all(user_id).await
    }

    async fn find_by_id(&self, id: &str) -> AppResult<Ticket> {
        self.repo.find_by_id(id).await
    }

    async fn create(&self, ticket: Ticket) -> AppResult<()> {
        self.repo.insert(ticket).await
    }

    async fn update(&self, id: &str, ticket: Ticket) -> AppResult<()> {
        self.repo.update(id, ticket).await
    }

    async fn delete(&self, id: &str) -> AppResult<()> {
        self.repo.delete(id).await
    }
}
