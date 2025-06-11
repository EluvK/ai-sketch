use salvo::{
    Depot, Request, Response, Router, Scribe, Writer, handler,
    http::cookie::{CookieBuilder, SameSite, time::Duration},
    oapi::{RouterExt, ToResponse, ToSchema, endpoint, extract::*},
    writing::Json,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use validator::Validate;

use crate::{
    app_data::AppDataRef,
    error::{ServiceError, ServiceResult},
    model::user::{User, UserRepository},
    utils::jwt::{generate_jwt_token, generate_refresh_token, verify_refresh_token},
};

pub fn create_router() -> Router {
    Router::new()
        .push(Router::with_path("logout").post(logout))
        .oapi_tag("auth")
    // .push(Router::with_path("edit").post(edit))
}

pub fn create_non_auth_router() -> Router {
    Router::new()
        .push(Router::with_path("phone-login").post(phone_login))
        .push(Router::with_path("refresh").post(refresh))
        .oapi_tag("auth")
    // .push(Router::with_path("register").post(register))
}

/// Phone Login
///
/// Authenticates a user using their phone number.
#[endpoint(
    status_codes(200, 201, 400, 401),
    request_body(content = PhoneLogin, description = "login/register by phone"),
    responses(
        (status_code = 200, body = LoginResult, description = "Successful login"),
        (status_code = 201, body = LoginResult, description = "User created and logged in"),
        (status_code = 400, description = "Bad Request: Validation error"),
        (status_code = 401, description = "Unauthorized: Invalid phone number or code")
    )
)]
async fn phone_login(
    login: JsonBody<PhoneLogin>,
    depot: &mut Depot,
    resp: &mut Response,
) -> ServiceResult<LoginResult> {
    // todo better validation
    login
        .validate()
        .map_err(|e| ServiceError::BadRequest(format!("Manually validation error: {}", e)))?;
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
            let new_user = User::new_by_phone(login.phone.clone());
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

    Ok(LoginResult {
        access_token,
        user_id,
    })
}

/// Refresh Token
///
/// Refreshes the JWT access token using the refresh token stored in cookies.
#[endpoint(
    status_codes(200, 401),
    responses(
        (status_code = 200, body = LoginResult, description = "Token refreshed successfully"),
        (status_code = 401, description = "Unauthorized: Refresh token not found or invalid")
    )
)]
async fn refresh(req: &mut Request, resp: &mut Response) -> ServiceResult<LoginResult> {
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

    Ok(LoginResult {
        access_token,
        user_id: user_id.clone(),
    })
}

/// Logout
///
/// Logs out the user by removing the refresh token cookie.
#[endpoint(
    status_codes(204, 401),
    responses(
        (status_code = 204, description = "Logout successful"),
        (status_code = 401, description = "Unauthorized: Refresh token not found")
    )
)]
async fn logout(req: &mut Request, depot: &mut Depot, resp: &mut Response) -> ServiceResult<()> {
    // todo(nice to have), cache the deprecated refresh token to make sure it can't be used again
    let _state = depot.obtain::<AppDataRef>()?;
    let user = depot.obtain::<User>()?;
    info!("Logging out user: {:?}", user);

    let refresh_token = req
        .cookies()
        .get("refresh_token")
        .ok_or_else(|| ServiceError::Unauthorized("Refresh token not found".to_string()))?
        .value();

    info!("Logging out user with refresh token: {}", refresh_token);
    resp.remove_cookie("refresh_token");
    resp.status_code(salvo::http::StatusCode::NO_CONTENT);
    Ok(())
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

/// Phone Login Request Body
#[derive(Serialize, Deserialize, ToSchema, Validate)]
#[serde(rename_all = "camelCase")]
struct PhoneLogin {
    #[validate(length(equal = 11))]
    #[salvo(schema(max_length = 11, min_length = 11, example = "139xxxx1234"))]
    phone: String,
    #[validate(length(equal = 6))]
    #[salvo(schema(max_length = 6, min_length = 6, example = "123456"))]
    code: String,
}

/// Login Response Body
#[derive(Serialize, Deserialize, ToResponse, ToSchema)]
#[serde(rename_all = "camelCase")]
struct LoginResult {
    #[salvo(schema(example = "jwt.token.here"))]
    access_token: String,
    #[salvo(schema(example = "user-uuid"))]
    user_id: String,
}

impl Scribe for LoginResult {
    fn render(self, res: &mut Response) {
        res.render(Json(self));
    }
}
