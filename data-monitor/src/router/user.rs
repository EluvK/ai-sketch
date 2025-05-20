use crate::{
    app_data::AppDataRef,
    error::ServiceResult,
    model::{User, UserRepository},
};

use salvo::{Depot, Response, Router, handler};

pub fn router() -> Router {
    Router::new().get(get_data)
}

#[handler]
fn get() {}

#[handler]
async fn get_data(depot: &mut Depot, res: &mut Response) -> ServiceResult<()> {
    let state = depot.obtain::<AppDataRef>()?;
    let new_user = User {
        id: "123".to_string(),
        name: "John Doe".to_string(),
        created_at: bson::DateTime::now(),
        last_login_at: bson::DateTime::now(),
    };
    state.mongo_client.create_user(new_user).await?;

    Ok(())
}
