use axum::{
    Router,
    routing::{get, post},
};

use sea_orm::{Database, DatabaseConnection};

use axum_login::{AuthManagerLayerBuilder, login_required};
use tower_sessions::{MemoryStore, SessionManagerLayer};
mod entities;
mod routes;

#[derive(Clone)]
pub struct DbState {
    db: DatabaseConnection,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().expect("Expected .env");
    tracing_subscriber::fmt::init();


    let database_url: String = std::env::var("DATABASE_URL").expect("database name missing");


    let state = DbState {
        db: Database::connect(&database_url).await?,
    };

    let session_layer = SessionManagerLayer::new(MemoryStore::default()).with_secure(false);
    let backend = routes::user::Backend::new(state.db.clone());
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    let protected = Router::new()        
        .route("/debug/me", get(routes::user::me))
        .route_layer(login_required!(routes::user::Backend, login_url ="/login"));
    
    let app = Router::new()
        .route("/", get(root))
        .route("/users", post(routes::user::create_user))
        .route("/login", post(routes::user::login))
        .route("/logout", post(routes::user::logout))
        .route("/product", get(routes::product::get_products))
        .merge(protected)
        .layer(auth_layer)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Starting server at port 3000");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> &'static str {
    "Landing"
}
