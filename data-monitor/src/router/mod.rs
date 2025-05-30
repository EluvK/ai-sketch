use chrono::NaiveDate;
use salvo::{Depot, Request, Response, Router, handler, writing::Text};
use serde::Deserialize;
use tracing::info;

use crate::{
    app_data::AppDataRef,
    error::{ServiceError, ServiceResult},
    model::{
        DailyStatisticsResponse, OverviewStatisticResponse, StatisticsRepository, StatisticsType,
    },
};

mod mock;

pub fn create_router() -> Router {
    Router::new()
        .push(
            Router::with_path("meter")
                .get(index_handler)
                .push(Router::with_path("daily").get(get_daily))
                .push(Router::with_path("overview").get(get_overview)),
        )
        .push(Router::with_path("mock").push(mock::router()))
}

#[handler]
async fn index_handler(res: &mut Response) {
    res.render(Text::Plain("Data Monitor is running."));
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetStatisticsRequest {
    pub r#type: StatisticsType,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

#[handler]
async fn get_daily(req: &mut Request, depot: &mut Depot) -> ServiceResult<DailyStatisticsResponse> {
    let body = req.parse_queries::<GetStatisticsRequest>()?;
    info!(
        "Querying statistics from {:?} to {:?}",
        body.start_date, body.end_date
    );
    let state = depot.obtain::<AppDataRef>()?;
    let now_date = chrono::Utc::now().date_naive();
    let mut daily_user = state
        .mongo_client
        .get_by_range(
            body.r#type,
            (
                body.start_date
                    .unwrap_or(now_date - chrono::Duration::days(7)),
                body.end_date.unwrap_or(now_date),
            ),
        )
        .await?;
    daily_user.sort_by(|a, b| a.date.cmp(&b.date));
    Ok(daily_user.try_into()?)
}

#[handler]
async fn get_overview(depot: &mut Depot) -> ServiceResult<OverviewStatisticResponse> {
    let state = depot.obtain::<AppDataRef>()?;
    let now_date = chrono::Utc::now().date_naive();
    let overview = state
        .mongo_client
        .get(now_date, StatisticsType::Overview)
        .await?
        .ok_or(ServiceError::NotFound(
            "No overview statistics found for the current date".to_string(),
        ))?;
    tracing::info!("Overview statistics: {:?}", overview);
    Ok(overview.try_into()?)
}
