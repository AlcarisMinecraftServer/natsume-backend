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
                SELECT id, category, materials, product, is_hidden, cooldown, unlock_level
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
                SELECT id, category, materials, product, is_hidden, cooldown, unlock_level
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
                materials: serde_json::from_value(row.get("materials")).unwrap_or_default(),
                product: serde_json::from_value(row.get("product")).unwrap(),
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
            SELECT id, category, materials, product, is_hidden, cooldown, unlock_level
            FROM recipes WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Recipe {
            id: row.get("id"),
            category: row.get("category"),
            materials: serde_json::from_value(row.get("materials")).unwrap_or_default(),
            product: serde_json::from_value(row.get("product")).unwrap(),
            is_hidden: row.get("is_hidden"),
            cooldown: row.get("cooldown"),
            unlock_level: row.get("unlock_level"),
        })
    }

    async fn insert(&self, recipe: Recipe) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO recipes (
                id, category, materials, product, is_hidden, cooldown, unlock_level
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7
            )
            "#,
        )
        .bind(&recipe.id)
        .bind(&recipe.category)
        .bind(serde_json::to_value(&recipe.materials)?)
        .bind(serde_json::to_value(&recipe.product)?)
        .bind(recipe.is_hidden)
        .bind(recipe.cooldown)
        .bind(recipe.unlock_level)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn patch(&self, id: &str, patch: Value) -> AppResult<()> {
        let patch_obj = patch
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Invalid JSON patch"))?;

        let mut query_builder = sqlx::QueryBuilder::new("UPDATE recipes SET ");
        let mut separated = query_builder.separated(", ");

        if let Some(val) = patch_obj.get("category") {
            separated.push("category = ");
            separated.push_bind_unseparated(val.as_str());
        }
        if let Some(val) = patch_obj.get("materials") {
            separated.push("materials = ");
            separated.push_bind_unseparated(val);
        }
        if let Some(val) = patch_obj.get("product") {
            separated.push("product = ");
            separated.push_bind_unseparated(val);
        }
        if let Some(val) = patch_obj.get("is_hidden") {
            separated.push("is_hidden = ");
            separated.push_bind_unseparated(val.as_bool());
        }
        if let Some(val) = patch_obj.get("cooldown") {
            separated.push("cooldown = ");
            separated.push_bind_unseparated(val.as_i64().map(|v| v as i32));
        }
        if let Some(val) = patch_obj.get("unlock_level") {
            separated.push("unlock_level = ");
            separated.push_bind_unseparated(val.as_i64().map(|v| v as i32));
        }

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(id);

        let query = query_builder.build();
        query.execute(&self.pool).await?;

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
