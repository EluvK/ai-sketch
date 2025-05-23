use ai_flow_synth::utils::MongoClient;
use bson::{DateTime, doc, to_bson};
use chrono::NaiveDate;
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};

use crate::model::constant::*;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum StatisticsType {
    Overview,
    DailyUserNumbers,
    DailyTokenUsage,
    DailyAmountNumbers,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Statistic {
    pub date: NaiveDate,
    pub r#type: StatisticsType,
    pub content: StatisticContent,
    pub update_time: DateTime,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum StatisticContent {
    Overview(OverviewStatistic),
    DailyNumbers(DailyStatistic),
    // todo 需要新的统计类型
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct OverviewStatistic {
    pub user_active_weekly: i64,
    pub user_summary: i64,
    pub token_usage_weekly: i64,
    pub token_usage_monthly: i64,
    pub amount_weekly: i64,
    pub amount_monthly: i64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DailyStatistic {
    pub increment: i64,
    pub total: i64,
    pub active: i64,
}

#[async_trait::async_trait]
pub trait StatisticsRepository {
    async fn upsert(&self, statistic: Statistic) -> anyhow::Result<u64>;
    async fn get(
        &self,
        date: NaiveDate,
        r#type: StatisticsType,
    ) -> anyhow::Result<Option<Statistic>>;
    async fn get_by_range(
        &self,
        r#type: StatisticsType,
        date_range: (NaiveDate, NaiveDate),
    ) -> anyhow::Result<Vec<Statistic>>;
}

#[async_trait::async_trait]
impl StatisticsRepository for MongoClient {
    async fn upsert(&self, statistic: Statistic) -> anyhow::Result<u64> {
        let collection = self.collection::<Statistic>(ADMIN_STATISTICS_COLLECTION_NAME);
        let filter =
            doc! { "date": to_bson(&statistic.date)?, "type": to_bson(&statistic.r#type)? };
        let update = doc! { SET_OP: bson::to_bson(&statistic)? };
        let options = mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build();
        let result = collection
            .update_one(filter, update)
            .with_options(options)
            .await?;
        Ok(result.modified_count)
    }
    async fn get(
        &self,
        date: NaiveDate,
        r#type: StatisticsType,
    ) -> anyhow::Result<Option<Statistic>> {
        let collection = self.collection::<Statistic>(ADMIN_STATISTICS_COLLECTION_NAME);
        let filter = doc! { "date": to_bson(&date)?, "type": to_bson(&r#type)? };
        let result = collection.find_one(filter).await?;
        Ok(result)
    }
    async fn get_by_range(
        &self,
        r#type: StatisticsType,
        date_range: (NaiveDate, NaiveDate),
    ) -> anyhow::Result<Vec<Statistic>> {
        let collection = self.collection::<Statistic>(ADMIN_STATISTICS_COLLECTION_NAME);
        let filter = doc! { "type": to_bson(&r#type)?, "date": { GTE_OP: to_bson(&date_range.0)?, LTE_OP: to_bson(&date_range.1)? } };
        let cursor = collection.find(filter).await?;
        let result: Vec<Statistic> = cursor.try_collect().await?;
        Ok(result)
    }
}
