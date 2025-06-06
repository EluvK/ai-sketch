use salvo::{Router, Writer, handler};

use crate::{
    app_data::AppDataRef,
    error::ServiceResult,
    model::user::{
        User, UserRepository,
        schema::{UpdateUserInfo, UserInfoResponse},
    },
};

pub fn create_router() -> Router {
    Router::new().push(
        Router::with_path("info")
            .get(get_user_info)
            .post(update_user_info),
    )
}

#[handler]
async fn get_user_info(depot: &mut salvo::Depot) -> ServiceResult<UserInfoResponse> {
    let user = depot.obtain::<User>()?;
    Ok(user.clone().into())
}

#[handler]
async fn update_user_info(info: UpdateUserInfo, depot: &mut salvo::Depot) -> ServiceResult<()> {
    let state = depot.obtain::<AppDataRef>()?;
    let current_user = depot.obtain::<User>()?;
    // todo update from info
    let new_user = User {
        ..current_user.clone()
    };
    state.mongo_client.update_user(new_user).await?;
    Ok(())
}
