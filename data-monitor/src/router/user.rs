use crate::{
    app_data::AppDataRef,
    error::{ServiceError, ServiceResult},
    model::{
        DailyStatistics, DailyStatisticsRepository, DailyStatisticsType, User, UserRepository,
    },
};

use bson::DateTime;
use salvo::{Depot, Request, Response, Router, handler};
use serde::Deserialize;

pub fn router() -> Router {
    Router::new().get(get_data)
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetDailyStatisticsRequest {
    pub r#type: DailyStatisticsType,
    pub start_date: Option<DateTime>,
    pub end_date: Option<DateTime>,
}

#[handler]
async fn get_data(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) -> ServiceResult<DailyStatistics> {
    let body = req.parse_json::<GetDailyStatisticsRequest>().await?;
    let state = depot.obtain::<AppDataRef>()?;

    let time = match (body.start_date, body.end_date) {
        (Some(start), Some(end)) => Some((start, end)),
        _ => None,
    };
    let statistics = state.mongo_client.get_by_type(body.r#type, time).await?;
    Ok(DailyStatistics(statistics))
}

#[handler]
async fn mock_create(depot: &mut Depot, res: &mut Response) -> ServiceResult<()> {
    let state = depot.obtain::<AppDataRef>()?;
    let new_user = User {
        id: uuid::Uuid::new_v4().to_string(),
        name: "test_user".to_string(),
        created_at: bson::DateTime::now(),
        last_login_at: bson::DateTime::now(),
    };
    state.mongo_client.create_user(new_user).await?;

    Ok(())
}
