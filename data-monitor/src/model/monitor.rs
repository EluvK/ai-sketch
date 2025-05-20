use std::vec;

use ai_flow_synth::utils::{IndexModel, MongoClient};
use bson::{doc, to_bson};
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};

use crate::model::constant::*;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DailyStatistics {
    pub date: String,
    pub r#type: DailyStatisticsType,
    pub increment: i64,
    pub total: i64,
    pub active: i64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum DailyStatisticsType {
    UserNumbers,
}

pub async fn create_index(client: &MongoClient) -> anyhow::Result<()> {
    let collection = client.collection::<DailyStatistics>(DAILY_STATISTICS_COLLECTION_NAME);
    let indexes = vec![
        IndexModel::builder().keys(doc! { "date": 1 }).build(),
        IndexModel::builder().keys(doc! { "type": 1 }).build(),
    ];
    collection.create_indexes(indexes).await?;
    Ok(())
}

#[async_trait::async_trait]
pub trait DailyStatisticsRepository {
    async fn insert(&self, statistics: DailyStatistics) -> anyhow::Result<()>;
    async fn get_by_date(&self, date: String) -> anyhow::Result<Vec<DailyStatistics>>;
    async fn get_by_type(
        &self,
        r#type: DailyStatisticsType,
    ) -> anyhow::Result<Vec<DailyStatistics>>;
    async fn get_by_date_and_type(
        &self,
        date: String,
        r#type: DailyStatisticsType,
    ) -> anyhow::Result<Vec<DailyStatistics>>;
}

#[async_trait::async_trait]
impl DailyStatisticsRepository for MongoClient {
    async fn insert(&self, statistics: DailyStatistics) -> anyhow::Result<()> {
        let collection = self.collection::<DailyStatistics>(DAILY_STATISTICS_COLLECTION_NAME);
        collection.insert_one(statistics).await?;
        Ok(())
    }

    async fn get_by_date(&self, date: String) -> anyhow::Result<Vec<DailyStatistics>> {
        let collection = self.collection::<DailyStatistics>(DAILY_STATISTICS_COLLECTION_NAME);
        let filter = doc! { "date": date };
        let cursor = collection.find(filter).await?;
        let results: Vec<DailyStatistics> = cursor.try_collect().await?;
        Ok(results)
    }

    async fn get_by_type(
        &self,
        r#type: DailyStatisticsType,
    ) -> anyhow::Result<Vec<DailyStatistics>> {
        let collection = self.collection::<DailyStatistics>(DAILY_STATISTICS_COLLECTION_NAME);
        let filter = doc! { "type": to_bson(&r#type)? };
        let cursor = collection.find(filter).await?;
        let results: Vec<DailyStatistics> = cursor.try_collect().await?;
        Ok(results)
    }

    async fn get_by_date_and_type(
        &self,
        date: String,
        r#type: DailyStatisticsType,
    ) -> anyhow::Result<Vec<DailyStatistics>> {
        let collection = self.collection::<DailyStatistics>(DAILY_STATISTICS_COLLECTION_NAME);
        let filter = doc! { "date": date, "type": to_bson(&r#type)? };
        let cursor = collection.find(filter).await?;
        let results: Vec<DailyStatistics> = cursor.try_collect().await?;
        Ok(results)
    }
}
