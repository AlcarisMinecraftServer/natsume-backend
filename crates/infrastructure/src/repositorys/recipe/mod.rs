use async_trait::async_trait;
use domain::recipes::Recipe;
use serde_json::Value;
use shared::error::AppResult;
use sqlx::{PgPool, Row};

#[async_trait]
pub trait RecipeRepository {
    async fn fetch_all(&self, category: Option<String>) -> AppResult<Vec<Recipe>>;
    async fn find_by_id(&self, id: &str) -> AppResult<Recipe>;
    async fn insert(&self, recipe: Recipe) -> AppResult<()>;
    async fn patch(&self, id: &str, patch: Value) -> AppResult<()>;
    async fn delete(&self, id: &str) -> AppResult<()>;
}

pub struct PostgresRecipeRepository {
    pub pool: PgPool,
}

impl PostgresRecipeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RecipeRepository for PostgresRecipeRepository {
    async fn fetch_all(&self, category: Option<String>) -> AppResult<Vec<Recipe>> {
        let rows = if let Some(category) = category {
            sqlx::query(
                r#"
                SELECT id, category, inputs, output, is_hidden, cooldown, unlock_level
                FROM recipes
                WHERE category = $1
                "#,
            )
            .bind(category)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                r#"
                SELECT id, category, inputs, output, is_hidden, cooldown, unlock_level
                FROM recipes
                "#,
            )
            .fetch_all(&self.pool)
            .await?
        };

        let recipes = rows
            .into_iter()
            .map(|row| Recipe {
                id: row.get("id"),
                category: row.get("category"),
                inputs: serde_json::from_value(row.get("inputs")).unwrap_or_default(),
                output: serde_json::from_value(row.get("output")).unwrap(),
                is_hidden: row.get("is_hidden"),
                cooldown: row.get("cooldown"),
                unlock_level: row.get("unlock_level"),
            })
            .collect();

        Ok(recipes)
    }

    async fn find_by_id(&self, id: &str) -> AppResult<Recipe> {
        let row = sqlx::query(
            r#"
            SELECT id, category, inputs, output, is_hidden, cooldown, unlock_level
            FROM recipes WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Recipe {
            id: row.get("id"),
            category: row.get("category"),
            inputs: serde_json::from_value(row.get("inputs")).unwrap_or_default(),
            output: serde_json::from_value(row.get("output")).unwrap(),
            is_hidden: row.get("is_hidden"),
            cooldown: row.get("cooldown"),
            unlock_level: row.get("unlock_level"),
        })
    }

    async fn insert(&self, recipe: Recipe) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO recipes (
                id, category, inputs, output, is_hidden, cooldown, unlock_level
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7
            )
            "#,
        )
        .bind(&recipe.id)
        .bind(&recipe.category)
        .bind(serde_json::to_value(&recipe.inputs)?)
        .bind(serde_json::to_value(&recipe.output)?)
        .bind(recipe.is_hidden)
        .bind(recipe.cooldown)
        .bind(recipe.unlock_level)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn patch(&self, id: &str, patch: Value) -> AppResult<()> {
        let patch_sql = "UPDATE recipes SET data = data || $1 WHERE id = $2";
        sqlx::query(patch_sql)
            .bind(patch)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete(&self, id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM recipes WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
