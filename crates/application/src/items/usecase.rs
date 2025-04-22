use async_trait::async_trait;
use serde_json::Value;

use domain::items::Item;
use infrastructure::repositorys::item::ItemRepository;
use shared::error::AppResult;

pub struct ItemUsecaseImpl<R: ItemRepository + Send + Sync> {
    pub repo: R,
}

impl<R: ItemRepository + Send + Sync> ItemUsecaseImpl<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

#[async_trait]
pub trait ItemUsecase: Send + Sync {
    async fn find_all(&self, category: Option<String>) -> AppResult<Vec<Item>>;
    async fn find_by_id(&self, id: &str) -> AppResult<Item>;
    async fn create(&self, item: Item) -> AppResult<()>;
    async fn patch(&self, id: &str, patch: Value) -> AppResult<()>;
    async fn delete(&self, id: &str) -> AppResult<()>;
}

#[async_trait]
impl<R: ItemRepository + Send + Sync> ItemUsecase for ItemUsecaseImpl<R> {
    async fn find_all(&self, category: Option<String>) -> AppResult<Vec<Item>> {
        self.repo.fetch_all(category).await
    }

    async fn find_by_id(&self, id: &str) -> AppResult<Item> {
        self.repo.find_by_id(id).await
    }

    async fn create(&self, item: Item) -> AppResult<()> {
        self.repo.insert(item).await
    }

    async fn patch(&self, id: &str, patch: Value) -> AppResult<()> {
        self.repo.patch(id, patch).await
    }

    async fn delete(&self, id: &str) -> AppResult<()> {
        self.repo.delete(id).await
    }
}
