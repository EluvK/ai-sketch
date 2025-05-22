use ai_flow_synth::utils::MongoClient;
use bson::{doc, to_bson};
use chrono::NaiveDate;
use futures_util::TryStreamExt;
use salvo::Scribe;
use serde::{Deserialize, Serialize};

use crate::model::constant::*;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OverviewStatistic {
    pub date: NaiveDate,
    pub user_summary: i64,
    pub user_active_weekly: i64,
    pub token_usage_summary: i64,
    pub token_usage_weekly: i64,
    pub amount_summary: i64,
    pub amount_weekly: i64,
}

impl Scribe for OverviewStatistic {
    fn render(self, res: &mut salvo::Response) {
        res.render(salvo::writing::Json(&self));
    }
}

#[async_trait::async_trait]
pub trait OverviewStatisticsRepository {
    async fn upsert(&self, statistics: OverviewStatistic) -> anyhow::Result<u64>;
    async fn get_by_date(&self, date: String) -> anyhow::Result<Option<OverviewStatistic>>;
    async fn get_by_date_range(
        &self,
        start_date: String,
        end_date: String,
    ) -> anyhow::Result<Vec<OverviewStatistic>>;
}

#[async_trait::async_trait]
impl OverviewStatisticsRepository for MongoClient {
    async fn upsert(&self, statistics: OverviewStatistic) -> anyhow::Result<u64> {
        let collection = self.collection::<OverviewStatistic>(ADMIN_STATISTICS_COLLECTION_NAME);
        let filter = doc! { "date": to_bson(&statistics.date)? };
        let update = doc! { SET_OP: bson::to_bson(&statistics)? };
        let options = mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build();
        let result = collection
            .update_one(filter, update)
            .with_options(options)
            .await?;
        Ok(result.modified_count)
    }

    async fn get_by_date(&self, date: String) -> anyhow::Result<Option<OverviewStatistic>> {
        let collection = self.collection::<OverviewStatistic>(ADMIN_STATISTICS_COLLECTION_NAME);
        let filter = doc! { "date": date };
        let result = collection.find_one(filter).await?;
        Ok(result)
    }
    async fn get_by_date_range(
        &self,
        start_date: String,
        end_date: String,
    ) -> anyhow::Result<Vec<OverviewStatistic>> {
        let collection = self.collection::<OverviewStatistic>(ADMIN_STATISTICS_COLLECTION_NAME);
        let filter = doc! { "date": { "$gte": start_date, "$lte": end_date } };
        let cursor = collection.find(filter).await?;
        let result: Vec<OverviewStatistic> = cursor.try_collect().await?;
        Ok(result)
    }
}
