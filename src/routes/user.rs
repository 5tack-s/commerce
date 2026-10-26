use std::{fmt, sync::OnceLock};

use argon2::{
    password_hash::{PasswordHasher, PasswordVerifier},
    Argon2, PasswordHash,
};
use axum::{extract::State, http::StatusCode, Json};
use axum_login::{AuthUser, AuthnBackend, UserId};
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set, SqlErr};
use serde::{Deserialize, Serialize};
use tokio::task::{spawn_blocking, JoinError};

use crate::entities::user;
use crate::DbState;

#[derive(Deserialize)]
pub struct CreateUser {
    username: String,
    email: String,
    password: String,
}

#[derive(Clone, Deserialize)]
pub struct Credentials {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct MessageResponse {
    message: String,
}

impl AuthUser for user::Model {
    type Id = i32;

    fn id(&self) -> i32 {
        self.id
    }

    // changing the password changes this, which logs out old sessions
    fn session_auth_hash(&self) -> &[u8] {
        self.password.as_bytes()
    }
}

#[derive(Debug)]
pub enum AuthError {
    Db(DbErr),
    Task(JoinError),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::Db(e) => write!(f, "database error: {e}"),
            AuthError::Task(e) => write!(f, "verify task failed: {e}"),
        }
    }
}

impl std::error::Error for AuthError {}

impl From<DbErr> for AuthError {
    fn from(e: DbErr) -> Self {
        AuthError::Db(e)
    }
}

impl From<JoinError> for AuthError {
    fn from(e: JoinError) -> Self {
        AuthError::Task(e)
    }
}

#[derive(Clone)]
pub struct Backend {
    db: DatabaseConnection,
}

impl Backend {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

static DUMMY_HASH: OnceLock<String> = OnceLock::new();

fn dummy_hash() -> &'static str {
    DUMMY_HASH.get_or_init(|| {
        Argon2::default()
            .hash_password(b"dummy-fuckass")
            .expect("dummy hashing failed")
            .to_string()
    })
}

impl AuthnBackend for Backend {
    type User = user::Model;
    type Credentials = Credentials;
    type Error = AuthError;

    async fn authenticate(&self, creds: Credentials) -> Result<Option<user::Model>, AuthError> {
        let found = user::Entity::find()
            .filter(user::Column::Email.eq(creds.email))
            .one(&self.db)
            .await?;

        let stored_hash = found.as_ref().map(|u| u.password.clone());
        let password = creds.password;

        let valid = spawn_blocking(move || {
            let hash_str = stored_hash.as_deref().unwrap_or_else(|| dummy_hash());
            match PasswordHash::new(hash_str) {
                Ok(parsed) => {
                    Argon2::default()
                        .verify_password(password.as_bytes(), &parsed)
                        .is_ok()
                        && stored_hash.is_some()
                }
                Err(_) => false,
            }
        })
        .await?;

        Ok(found.filter(|_| valid))
    }

    async fn get_user(&self, id: &UserId<Self>) -> Result<Option<user::Model>, AuthError> {
        Ok(user::Entity::find_by_id(*id).one(&self.db).await?)
    }
}

pub type AuthSession = axum_login::AuthSession<Backend>;

pub async fn create_user(
    State(state): State<DbState>,
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<MessageResponse>) {
    let hashed = match spawn_blocking(move || {
        let argon2 = Argon2::default();
        argon2
            .hash_password(payload.password.as_bytes())
            .map(|h| h.to_string())
            .map(|h| (h, payload.email, payload.username))
    })
    .await
    {
        Ok(Ok(v)) => v,
        Ok(Err(_)) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MessageResponse {
                    message: "Hashing failed.".to_string(),
                }),
            )
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MessageResponse {
                    message: "Hashing task failed.".to_string(),
                }),
            )
        }
    };
    let (hash, email, username) = hashed;

    let new_user = user::ActiveModel {
        email: Set(email),
        username: Set(username),
        password: Set(hash),
        ..Default::default()
    };

    match user::Entity::insert(new_user).exec(&state.db).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(MessageResponse {
                message: "User created".to_string(),
            }),
        ),
        Err(e) => match e.sql_err() {
            Some(SqlErr::UniqueConstraintViolation(_)) => (
                StatusCode::CONFLICT,
                Json(MessageResponse {
                    message: "Username or email already taken".to_string(),
                }),
            ),
            _ => {
                tracing::error!("User registration failed! {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(MessageResponse {
                        message: "User registration failed!".to_string(),
                    }),
                )
            }
        },
    }
}

pub async fn login(
    mut auth: AuthSession,
    Json(creds): Json<Credentials>,
) -> Result<Json<MessageResponse>, (StatusCode, Json<MessageResponse>)> {
    let unauthorized = || {
        (
            StatusCode::UNAUTHORIZED,
            Json(MessageResponse {
                message: "Wrong credentials.".to_string(),
            }),
        )
    };
    let server_error = || {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MessageResponse {
                message: "Login failed".to_string(),
            }),
        )
    };

    let user = auth
        .authenticate(creds)
        .await
        .map_err(|e| {
            tracing::error!("login authenticate error: {e}");
            server_error()
        })?
        .ok_or_else(unauthorized)?;

    auth.login(&user).await.map_err(|e| {
        tracing::error!("login session error: {e}");
        server_error()
    })?;

    Ok(Json(MessageResponse {
        message: "Logged in".to_string(),
    }))
}

pub async fn logout(mut auth: AuthSession) -> StatusCode {
    match auth.logout().await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            tracing::error!("logout error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}


pub async fn me(auth: AuthSession) -> Json<MessageResponse> {
    let user = auth.user.expect("login_required! guarantees a user");
    Json(MessageResponse {
        message: format!("hello {}", user.username),
    })
}