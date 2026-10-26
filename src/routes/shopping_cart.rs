use axum::{Json, extract::State, http::StatusCode};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::{DbState, entities::product};


#[derive(Deserialize)]
pub struct CartOperation{
    id: i32,
    quantity: i32
}

struct MessageResponse {
    message: String
}

#[derive(Serialize)]
struct Payload {
    product_id: i32,
    amount: i32
}

pub async fn take_product(
    State(state): State<DbState>,
    Json(payload): Json<Payload>
) -> Result<Json<CartOperation>, (StatusCode, Json<MessageResponse>)>{

    let product = product::Entity::find_by_id(payload.product_id)
    .one(&state.db)
    .await
    .map_err(|e|{
        tracing::error!("Failed to get a column from product! {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, Json(MessageResponse{message: "Failed to fetch product columns from database.".into()}))
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(MessageResponse {
                message: "product not found".to_string(),
            }),
        )
    })?;    
    

    Ok(Json(CartOperation{
        id: product.id,
        quantity: payload.amount
    }))

} 