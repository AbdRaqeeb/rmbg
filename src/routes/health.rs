use actix_web::{get, Responder};

#[get("/")]
pub async fn index() -> impl Responder {
    "healthy".to_string()
}
