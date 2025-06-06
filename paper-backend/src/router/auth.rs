use salvo::{
    Depot, Request, Response, Router, Scribe, Writer, handler,
    http::cookie::{CookieBuilder, SameSite, time::Duration},
    macros::Extractible,
    writing::Json,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    app_data::AppDataRef,
    error::{ServiceError, ServiceResult},
    model::user::{User, UserRepository},
    utils::jwt::{generate_jwt_token, generate_refresh_token, verify_refresh_token},
};

pub fn create_router() -> Router {
    Router::new().push(Router::with_path("edit").post(edit))
}

pub fn create_non_auth_router() -> Router {
    Router::new()
        .push(Router::with_path("phone-login").post(phone_login))
        .push(Router::with_path("register").post(register))
        .push(Router::with_path("refresh").post(refresh))
}

#[handler]
async fn phone_login(
    login: PhoneLogin,
    depot: &mut Depot,
    resp: &mut Response,
) -> ServiceResult<LoginResponse> {
    let state = depot.obtain::<AppDataRef>()?;
    let exist_user = state.mongo_client.get_user_by_phone(&login.phone).await?;
    let user_id = match exist_user {
        Some(user) => {
            // Here you would typically verify the code sent to the user's phone
            // For simplicity, we assume the code is valid
            tracing::info!("User found: {:?}", user);
            user.uid
        }
        None => {
            tracing::info!("create new user found with phone: {}", login.phone);
            let new_user = User::new_by_phone(login.phone);
            let user_id = new_user.uid.clone();
            state.mongo_client.create_user(new_user).await?;
            resp.status_code(salvo::http::StatusCode::CREATED);
            user_id
        }
    };

    let access_token = generate_jwt_token(user_id.clone())?;
    let refresh_token = generate_refresh_token(user_id.clone())?;
    resp.add_cookie(
        CookieBuilder::new("refresh_token", refresh_token)
            .max_age(Duration::days(30))
            .same_site(SameSite::Lax)
            .http_only(true)
            .secure(false) // todo true in production
            .build(),
    );

    Ok(LoginResponse {
        access_token,
        user_id,
    })
}

#[handler]
async fn refresh(req: &mut Request, resp: &mut Response) -> ServiceResult<LoginResponse> {
    let refresh_token = req
        .cookies()
        .get("refresh_token")
        .ok_or_else(|| ServiceError::Unauthorized("Refresh token not found".to_string()))?
        .value();
    let user_id = verify_refresh_token(refresh_token)?.sub;

    info!("Refreshing token for user: {:?}", user_id);
    let access_token = generate_jwt_token(user_id.clone())?;
    let refresh_token = generate_refresh_token(user_id.clone())?;

    resp.add_cookie(
        CookieBuilder::new("refresh_token", refresh_token)
            .max_age(Duration::days(30))
            .same_site(SameSite::Lax)
            .http_only(true)
            .secure(false) // todo true in production
            .build(),
    );

    Ok(LoginResponse {
        access_token,
        user_id: user_id.clone(),
    })
}

#[handler]
async fn register(req: &mut Request, depot: &mut Depot, resp: &mut Response) -> ServiceResult<()> {
    let state = depot.obtain::<AppDataRef>()?;

    resp.status_code(salvo::http::StatusCode::CREATED);
    Ok(())
}

#[handler]
async fn edit(req: &mut Request, depot: &mut Depot, resp: &mut Response) -> ServiceResult<()> {
    let state = depot.obtain::<AppDataRef>()?;
    let user = depot.obtain::<User>()?;

    info!("Editing user: {:?}", user);
    Ok(())
}

#[derive(Serialize, Deserialize, Extractible)]
#[salvo(extract(default_source(from = "body")))]
struct PhoneLogin {
    phone: String,
    code: String,
}

#[derive(Serialize, Deserialize)]
struct LoginResponse {
    access_token: String,
    user_id: String,
}

impl Scribe for LoginResponse {
    fn render(self, res: &mut Response) {
        res.render(Json(self));
    }
}
