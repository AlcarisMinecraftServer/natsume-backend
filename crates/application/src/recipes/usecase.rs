use async_trait::async_trait;
use serde_json::Value;

use domain::recipes::Recipe;
use infrastructure::repositorys::recipe::RecipeRepository;
use shared::error::AppResult;

pub struct RecipeUsecaseImpl<R: RecipeRepository + Send + Sync> {
    pub repo: R,
}

impl<R: RecipeRepository + Send + Sync> RecipeUsecaseImpl<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

#[async_trait]
pub trait RecipeUsecase: Send + Sync {
    async fn find_all(&self, category: Option<String>) -> AppResult<Vec<Recipe>>;
    async fn find_by_id(&self, id: &str) -> AppResult<Recipe>;
    async fn create(&self, recipe: Recipe) -> AppResult<()>;
    async fn patch(&self, id: &str, patch: Value) -> AppResult<()>;
    async fn delete(&self, id: &str) -> AppResult<()>;
}

#[async_trait]
impl<R: RecipeRepository + Send + Sync> RecipeUsecase for RecipeUsecaseImpl<R> {
    async fn find_all(&self, category: Option<String>) -> AppResult<Vec<Recipe>> {
        self.repo.fetch_all(category).await
    }

    async fn find_by_id(&self, id: &str) -> AppResult<Recipe> {
        self.repo.find_by_id(id).await
    }

    async fn create(&self, recipe: Recipe) -> AppResult<()> {
        self.repo.insert(recipe).await
    }

    async fn patch(&self, id: &str, patch: Value) -> AppResult<()> {
        self.repo.patch(id, patch).await
    }

    async fn delete(&self, id: &str) -> AppResult<()> {
        self.repo.delete(id).await
    }
}
