use salvo::{
    Depot, FlowCtrl, Request, Response, Router,
    jwt_auth::{ConstDecoder, HeaderFinder, QueryFinder},
    prelude::{JwtAuth, JwtAuthDepotExt, JwtAuthState},
};

use crate::{
    app_data::AppDataRef,
    config::BackendConfig,
    error::{ServiceError, ServiceResult},
    model::user::UserRepository,
    utils::jwt::JwtClaims,
};

mod auth;
mod user;

pub fn create_router(config: &BackendConfig) -> Router {
    let auth_handler: JwtAuth<JwtClaims, _> = JwtAuth::new(ConstDecoder::from_secret(
        config.jwt.access_secret.as_bytes(),
    ))
    .finders(vec![
        Box::new(HeaderFinder::new()),
        Box::new(QueryFinder::new("jwt_token")),
    ])
    .force_passed(true);

    let non_auth_router =
        Router::new().push(Router::with_path("auth").push(auth::create_non_auth_router()));
    let auth_router = Router::new()
        .hoop(auth_handler)
        .hoop(jwt_to_user)
        .push(Router::with_path("auth").push(auth::create_router()))
        .push(Router::with_path("user").push(user::create_router()));

    Router::new().push(non_auth_router).push(auth_router)
}

#[salvo::handler]
async fn jwt_to_user(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
    ctrl: &mut FlowCtrl,
) -> ServiceResult<()> {
    match (depot.jwt_auth_state(), depot.jwt_auth_data::<JwtClaims>()) {
        (JwtAuthState::Authorized, Some(jwt_token)) => {
            tracing::info!("JWT is authorized");
            let claim = jwt_token.claims.clone();
            if claim.is_expired() {
                tracing::info!("JWT is expired");
                res.render(ServiceError::Unauthorized("JWT is expired".to_string()));
                ctrl.skip_rest();
            }
            let state = depot.obtain::<AppDataRef>()?;
            let user = state.mongo_client.get_user_by_uid(&claim.sub).await?;
            let Some(user) = user else {
                tracing::info!("Invalid user id: {}", claim.sub);
                res.render(ServiceError::Unauthorized("User not found".to_string()));
                ctrl.skip_rest();
                return Ok(());
            };
            depot.inject(user);
            // depot.insert(DEPOT_USER, user);
            ctrl.call_next(req, depot, res).await;
        }
        (_, None) => {
            tracing::info!("JWT is not provided");
            res.render(ServiceError::Unauthorized(
                "JWT token not provided".to_string(),
            ));
            ctrl.skip_rest();
        }
        _ => {
            tracing::info!("JWT is unauthorized");
            res.render(ServiceError::Unauthorized("JWT Unauthorized".to_string()));
            ctrl.skip_rest();
        }
    }
    Ok(())
}
