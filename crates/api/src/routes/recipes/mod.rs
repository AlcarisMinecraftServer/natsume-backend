use std::sync::Arc;

use application::recipes::RecipeUsecase;
use axum::{
    Extension, Json, extract::Path, extract::Query, http::StatusCode, response::IntoResponse,
};
use domain::{recipes::Recipe, response::ApiResponse};
use serde_json::Value;

#[derive(Debug, serde::Deserialize)]
pub struct ListRecipeQuery {
    pub category: Option<String>,
}

pub async fn find_all_recipes(
    Extension(usecase): Extension<Arc<dyn RecipeUsecase>>,
    Query(query): Query<ListRecipeQuery>,
) -> impl IntoResponse {
    match usecase.find_all(query.category).await {
        Ok(recipes) => Json(ApiResponse {
            status: 200,
            data: recipes,
        })
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": 500,
                "code": "fetch_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

pub async fn find_recipes_by_id(
    Extension(usecase): Extension<Arc<dyn RecipeUsecase>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match usecase.find_by_id(&id).await {
        Ok(recipe) => Json(ApiResponse {
            status: 200,
            data: recipe,
        })
        .into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "status": 404,
                "code": "not_found",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

pub async fn create_recipe(
    Extension(usecase): Extension<Arc<dyn RecipeUsecase>>,
    Json(recipe): Json<Recipe>,
) -> impl IntoResponse {
    match usecase.create(recipe).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(serde_json::json!({"status": 201})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": 500,
                "code": "create_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

pub async fn patch_recipe(
    Extension(usecase): Extension<Arc<dyn RecipeUsecase>>,
    Path(id): Path<String>,
    Json(patch): Json<Value>,
) -> impl IntoResponse {
    match usecase.patch(&id, patch).await {
        Ok(_) => Json(serde_json::json!({
            "status": 200,
            "message": "Recipe updated"
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": 500,
                "code": "update_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

pub async fn delete_recipe(
    Extension(usecase): Extension<Arc<dyn RecipeUsecase>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match usecase.delete(&id).await {
        Ok(_) => Json(serde_json::json!({
            "status": 200,
            "message": "Recipe deleted"
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": 500,
                "code": "delete_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}
