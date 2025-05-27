use async_trait::async_trait;
use domain::items::{Item, ItemCategory};
use serde_json::{Value, from_value, to_value};
use shared::error::AppResult;
use sqlx::{PgPool, Row};

#[async_trait]
pub trait ItemRepository {
    async fn fetch_all(&self, category: Option<String>) -> AppResult<Vec<Item>>;
    async fn find_by_id(&self, id: &str) -> AppResult<Item>;
    async fn insert(&self, item: Item) -> AppResult<()>;
    async fn patch(&self, id: &str, patch: Value) -> AppResult<()>;
    async fn delete(&self, id: &str) -> AppResult<()>;
}

pub struct PostgresItemRepository {
    pub pool: PgPool,
}

impl PostgresItemRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ItemRepository for PostgresItemRepository {
    async fn fetch_all(&self, category: Option<String>) -> AppResult<Vec<Item>> {
        let rows = sqlx::query("SELECT * FROM items")
            .fetch_all(&self.pool)
            .await?;

        let mut items = Vec::new();

        for row in rows {
            let category_str: String = row.get("category");
            if let Some(ref filter) = category {
                if category_str.to_lowercase() != filter.to_lowercase() {
                    continue;
                }
            }

            let item = Item {
                id: row.get("id"),
                version: row.get("version"),
                name: row.get("name"),
                category: match category_str.to_lowercase().as_str() {
                    "weapon" => ItemCategory::Weapon,
                    "tool" => ItemCategory::Tool,
                    "material" => ItemCategory::Material,
                    "food" => ItemCategory::Food,
                    "armor" => ItemCategory::Armor,
                    _ => continue,
                },
                lore: serde_json::from_value(row.get::<Value, _>("lore"))?,
                rarity: row.get("rarity"),
                max_stack: row.get("max_stack"),
                custom_model_data: row.get("custom_model_data"),
                price: serde_json::from_value(row.get("price"))?,
                tags: serde_json::from_value(row.get::<Value, _>("tags"))?,
                data: row.get("data"),
            };

            items.push(item);
        }

        Ok(items)
    }

    async fn find_by_id(&self, id: &str) -> AppResult<Item> {
        let row = sqlx::query("SELECT * FROM items WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        let category_str: String = row.get("category");

        Ok(Item {
            id: row.get("id"),
            version: row.get("version"),
            name: row.get("name"),
            category: match category_str.to_lowercase().as_str() {
                "weapon" => ItemCategory::Weapon,
                "tool" => ItemCategory::Tool,
                "material" => ItemCategory::Material,
                "food" => ItemCategory::Food,
                "armor" => ItemCategory::Armor,
                _ => return Err(anyhow::anyhow!("Invalid category")),
            },
            lore: serde_json::from_value(row.get::<Value, _>("lore"))?,
            rarity: row.get("rarity"),
            max_stack: row.get("max_stack"),
            custom_model_data: row.get("custom_model_data"),
            price: serde_json::from_value(row.get("price"))?,
            tags: serde_json::from_value(row.get::<Value, _>("tags"))?,
            data: row.get("data"),
        })
    }

    async fn insert(&self, item: Item) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO items (
                id, version, name, category,
                lore, rarity, max_stack, custom_model_data,
                price, tags, data
            ) VALUES (
                $1, $2, $3, $4,
                to_jsonb($5), $6, $7, $8,
                to_jsonb($9), to_jsonb($10), to_jsonb($11)
            )
            "#,
        )
        .bind(&item.id)
        .bind(item.version)
        .bind(&item.name)
        .bind(item.category.to_string())
        .bind(serde_json::to_value(&item.lore)?)
        .bind(item.rarity)
        .bind(item.max_stack)
        .bind(item.custom_model_data)
        .bind(serde_json::to_value(&item.price)?)
        .bind(serde_json::to_value(&item.tags)?)
        .bind(item.data)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn patch(&self, id: &str, patch: Value) -> AppResult<()> {
        let item: Item = from_value(patch)?;

        sqlx::query(
            r#"
            UPDATE items SET
                version = $1,
                name = $2,
                category = $3,
                lore = to_jsonb($4),
                rarity = $5,
                max_stack = $6,
                custom_model_data = $7,
                price = to_jsonb($8),
                tags = to_jsonb($9),
                data = to_jsonb($10)
            WHERE id = $11
            "#,
        )
        .bind(item.version)
        .bind(&item.name)
        .bind(item.category.to_string())
        .bind(to_value(&item.lore)?)
        .bind(item.rarity)
        .bind(item.max_stack)
        .bind(item.custom_model_data)
        .bind(to_value(&item.price)?)
        .bind(to_value(&item.tags)?)
        .bind(item.data)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM items WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
