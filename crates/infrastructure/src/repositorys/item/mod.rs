use async_trait::async_trait;
use domain::items::{CustomModelData, Item, ItemCategory};
use serde_json::Value;
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
        let category_filter = category.map(|c| c.to_lowercase());

        let query = r#"
            SELECT * FROM items 
            WHERE $1::text IS NULL OR category = $1
        "#;

        let rows: Vec<sqlx::postgres::PgRow> = sqlx::query(query)
            .bind(category_filter)
            .fetch_all(&self.pool)
            .await?;

        let mut items = Vec::new();

        for row in rows {
            let category_str: String = row.try_get("category")?;
            let item_category = match category_str.as_str() {
                "weapon" => ItemCategory::Weapon,
                "tool" => ItemCategory::Tool,
                "material" => ItemCategory::Material,
                "food" => ItemCategory::Food,
                "armor" => ItemCategory::Armor,
                _ => continue,
            };

            let cmd_value: Option<Value> = row.try_get("custom_model_data")?;
            let custom_model_data: Option<CustomModelData> = match cmd_value {
                None => None,
                Some(Value::Null | Value::Number(_)) => None,
                Some(other) => serde_json::from_value(other)?,
            };

            let item = Item {
                id: row.try_get("id")?,
                version: row.try_get("version")?,
                name: row.try_get("name")?,
                category: item_category,
                lore: serde_json::from_value(row.try_get::<Value, _>("lore")?)?,
                rarity: row.try_get("rarity")?,
                max_stack: row.try_get("max_stack")?,
                custom_model_data,
                item_model: row.try_get("item_model")?,
                tooltip_style: row.try_get("tooltip_style")?,
                price: serde_json::from_value(row.try_get("price")?)?,
                tags: serde_json::from_value(row.try_get::<Value, _>("tags")?)?,
                data: row.try_get("data")?,
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

        let category_str: String = row.try_get("category")?;
        let cmd_value: Option<Value> = row.try_get("custom_model_data")?;
        let custom_model_data: Option<CustomModelData> = match cmd_value {
            None => None,
            Some(Value::Null | Value::Number(_)) => None,
            Some(other) => serde_json::from_value(other)?,
        };

        Ok(Item {
            id: row.try_get("id")?,
            version: row.try_get("version")?,
            name: row.try_get("name")?,
            category: match category_str.to_lowercase().as_str() {
                "weapon" => ItemCategory::Weapon,
                "tool" => ItemCategory::Tool,
                "material" => ItemCategory::Material,
                "food" => ItemCategory::Food,
                "armor" => ItemCategory::Armor,
                _ => return Err(anyhow::anyhow!("Invalid category")),
            },
            lore: serde_json::from_value(row.try_get::<Value, _>("lore")?)?,
            rarity: row.try_get("rarity")?,
            max_stack: row.try_get("max_stack")?,
            custom_model_data,
            item_model: row.try_get("item_model")?,
            tooltip_style: row.try_get("tooltip_style")?,
            price: serde_json::from_value(row.try_get("price")?)?,
            tags: serde_json::from_value(row.try_get::<Value, _>("tags")?)?,
            data: row.try_get("data")?,
        })
    }

    async fn insert(&self, item: Item) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO items (
                id, version, name, category,
                lore, rarity, max_stack, custom_model_data,
                price, tags, data, item_model, tooltip_style
            ) VALUES (
                $1, $2, $3, $4,
                to_jsonb($5), $6, $7, to_jsonb($8),
                to_jsonb($9), to_jsonb($10), to_jsonb($11),
                $12, $13
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
        .bind(serde_json::to_value(&item.custom_model_data)?)
        .bind(serde_json::to_value(&item.price)?)
        .bind(serde_json::to_value(&item.tags)?)
        .bind(item.data)
        .bind(&item.item_model)
        .bind(&item.tooltip_style)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn patch(&self, id: &str, patch: Value) -> AppResult<()> {
        let patch_obj = patch
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Invalid JSON patch"))?;

        let mut query_builder = sqlx::QueryBuilder::new("UPDATE items SET ");
        let mut separated = query_builder.separated(", ");

        if let Some(val) = patch_obj.get("name") {
            separated.push("name = ");
            separated.push_bind_unseparated(val.as_str());
        }
        if let Some(val) = patch_obj.get("version") {
            separated.push("version = ");
            separated.push_bind_unseparated(val.as_i64());
        }
        if let Some(val) = patch_obj.get("category") {
            separated.push("category = ");
            separated.push_bind_unseparated(val.as_str());
        }
        if let Some(val) = patch_obj.get("lore") {
            separated.push("lore = ");
            separated.push_bind_unseparated(val);
        }
        if let Some(val) = patch_obj.get("rarity") {
            separated.push("rarity = ");
            separated.push_bind_unseparated(val.as_i64());
        }
        if let Some(val) = patch_obj.get("max_stack") {
            separated.push("max_stack = ");
            separated.push_bind_unseparated(val.as_i64());
        }
        if let Some(val) = patch_obj.get("custom_model_data") {
            separated.push("custom_model_data = ");
            separated.push_bind_unseparated(val);
        }
        if let Some(val) = patch_obj.get("price") {
            separated.push("price = ");
            separated.push_bind_unseparated(val);
        }
        if let Some(val) = patch_obj.get("tags") {
            separated.push("tags = ");
            separated.push_bind_unseparated(val);
        }
        if let Some(val) = patch_obj.get("data") {
            separated.push("data = ");
            separated.push_bind_unseparated(val);
        }
        if let Some(val) = patch_obj.get("item_model") {
            separated.push("item_model = ");
            separated.push_bind_unseparated(val.as_str());
        }
        if let Some(val) = patch_obj.get("tooltip_style") {
            separated.push("tooltip_style = ");
            separated.push_bind_unseparated(val.as_str());
        }

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(id);

        let query = query_builder.build();
        query.execute(&self.pool).await?;

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
