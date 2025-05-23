mod common;
mod daily_statistic;
mod overview_statistic;

use ai_flow_synth::utils::MongoClient;
use bson::doc;
use mongodb::IndexModel;

pub use common::*;
pub use daily_statistic::DailyStatisticsResponse;
pub use overview_statistic::OverviewStatisticResponse;

use super::constant::ADMIN_STATISTICS_COLLECTION_NAME;

pub async fn create_index(client: &MongoClient) -> anyhow::Result<()> {
    let collection = client.collection::<Statistic>(ADMIN_STATISTICS_COLLECTION_NAME);
    let indexes = vec![
        IndexModel::builder().keys(doc! { "date": 1 }).build(),
        IndexModel::builder().keys(doc! { "type": 1 }).build(),
    ];
    collection.create_indexes(indexes).await?;
    Ok(())
}
