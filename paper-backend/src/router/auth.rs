use salvo::{
    Depot, Request, Response, Router, Scribe, Writer, handler, macros::Extractible, writing::Json,
};
use serde::{Deserialize, Serialize};

use crate::{app_data::AppDataRef, error::ServiceResult, utils::jwt::generate_jwt_token};

pub fn create_router() -> Router {
    Router::new().push(Router::with_path("edit").post(edit))
}

pub fn create_non_auth_router() -> Router {
    Router::new()
        .push(Router::with_path("phone-login").post(phone_login))
        .push(Router::with_path("register").post(register))
}

#[handler]
async fn phone_login(login: PhoneLogin) -> ServiceResult<LoginResponse> {
    // Placeholder for phone-login logic

    let user_id: String = "example_user_id".to_owned(); // This should be replaced with actual user ID retrieval logic

    let access_token = generate_jwt_token(user_id.clone())?;
    Ok(LoginResponse {
        access_token,
        user_id,
    })
}

#[handler]
async fn register(req: &mut Request, depot: &mut Depot, resp: &mut Response) -> ServiceResult<()> {
    let state = depot.obtain::<AppDataRef>()?;

    resp.status_code(salvo::http::StatusCode::CREATED);
    Ok(())
}

#[handler]
async fn edit() -> &'static str {
    // Placeholder for edit logic
    "Edit successful"
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
