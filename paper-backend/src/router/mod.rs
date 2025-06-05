use salvo::{
    Depot, FlowCtrl, Request, Response, Router,
    jwt_auth::{ConstDecoder, HeaderFinder, QueryFinder},
    prelude::{JwtAuth, JwtAuthDepotExt, JwtAuthState},
};

use crate::{config::BackendConfig, error::ServiceError, utils::jwt::JwtClaims};

mod auth;

pub fn create_router(config: &BackendConfig) -> Router {
    let auth_handler: JwtAuth<JwtClaims, _> =
        JwtAuth::new(ConstDecoder::from_secret(config.jwt_secret.as_bytes()))
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
        .push(Router::with_path("auth").push(auth::create_router()));

    Router::new().push(non_auth_router).push(auth_router)
}

#[salvo::handler]
async fn jwt_to_user(
    req: &mut Request,
    res: &mut Response,
    depot: &mut Depot,
    ctrl: &mut FlowCtrl,
) {
    match (depot.jwt_auth_state(), depot.jwt_auth_data::<JwtClaims>()) {
        (JwtAuthState::Authorized, Some(jwt_token)) => {
            tracing::info!("JWT is authorized");
            let claim = jwt_token.claims.clone();
            if claim.is_expired() {
                tracing::info!("JWT is expired");
                res.render(ServiceError::Unauthorized("JWT is expired".to_string()));
                ctrl.skip_rest();
            }
            // todo: query user from database and set to depot
            ctrl.call_next(req, depot, res).await;
        }
        _ => {
            tracing::info!("JWT is not provided");
            res.render(ServiceError::Unauthorized(
                "JWT is not provided".to_string(),
            ));
            ctrl.skip_rest();
            tracing::info!("JWT is not authorized");
            tracing::info!("jwt token: {:?}", depot.jwt_auth_token());
            tracing::info!("jwt data: {:?}", depot.jwt_auth_data::<JwtClaims>());
            tracing::info!("jwt error: {:?}", depot.jwt_auth_error());
        }
    }
}
