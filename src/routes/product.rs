use axum::{Json, extract::State, http::StatusCode};
use sea_orm::{EntityTrait};
use serde_json::Value;
use crate::{DbState, entities::product};


pub async fn get_products(State(state): State<DbState>) -> Result<Json<Vec<Value>>, StatusCode> {
    product::Entity::find()
        .into_json()
        .all(&state.db)
        .await
        .map(Json)
        .map_err(|e| {
            tracing::error!("Database error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}