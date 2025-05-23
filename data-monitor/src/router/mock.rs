use crate::{
    app_data::AppDataRef,
    error::ServiceResult,
    model::{User, UserRepository},
};

use salvo::{Depot, Router, handler};

pub fn router() -> Router {
    Router::new().push(Router::with_path("user").get(create_user))
}

#[handler]
async fn create_user(depot: &mut Depot) -> ServiceResult<()> {
    let state = depot.obtain::<AppDataRef>()?;
    let days_ago: i64 = rand::random_range(1..=20);
    let created_at = chrono::Utc::now() - chrono::Duration::days(days_ago);
    let last_login_offset = rand::random_range(1..=days_ago);
    let last_login_at = created_at + chrono::Duration::days(last_login_offset);
    let new_user = User {
        id: uuid::Uuid::new_v4().to_string(),
        name: format!("user_{}", rand::random::<u32>()),
        created_at: bson::DateTime::from_chrono(created_at),
        last_login_at: bson::DateTime::from_chrono(last_login_at),
    };
    state.mongo_client.create_user(new_user).await?;
    Ok(())
}
