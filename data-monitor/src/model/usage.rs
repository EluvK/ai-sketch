use ai_flow_synth::utils::MongoClient;
use async_trait::async_trait;
use bson::{DateTime, doc};
use futures_util::TryStreamExt;
use mongodb::IndexModel;
use serde::{Deserialize, Serialize};

use crate::model::constant::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsageRecord {
    pub user_id: String,
    pub provider: String,
    pub llm_model: String,
    pub token_cost: f64, //?why f64
    pub usage_date: DateTime,
    pub price: f64, // per Million tokens
}

pub async fn create_index(client: &MongoClient) -> anyhow::Result<()> {
    let collection = client.collection::<UsageRecord>(USAGE_RECORD_COLLECTION_NAME);
    let user_date_index = IndexModel::builder()
        .keys(doc! { "user_id": 1, "usage_date": 1 })
        .build();
    let date_index = IndexModel::builder().keys(doc! { "usage_date": 1 }).build();
    collection
        .create_indexes(vec![user_date_index, date_index])
        .await?;
    Ok(())
}

#[async_trait]
pub trait UsageRecordRepository {
    async fn save_usage_record(&self, record: UsageRecord) -> anyhow::Result<()>;

    async fn get_usage_records_by_date_range(
        &self,
        date_range: (DateTime, DateTime),
    ) -> anyhow::Result<Vec<UsageRecord>>;

    async fn fold_usage_records_by_date_range<T, F>(
        &self,
        date_range: (DateTime, DateTime),
        init: T,
        mut f: F,
    ) -> anyhow::Result<T>
    where
        T: Send + Sync,
        F: Send + Sync + FnMut(T, UsageRecord) -> T;
}

#[async_trait]
impl UsageRecordRepository for MongoClient {
    async fn save_usage_record(&self, record: UsageRecord) -> anyhow::Result<()> {
        let collection = self.collection::<UsageRecord>(USAGE_RECORD_COLLECTION_NAME);
        collection.insert_one(record).await?;
        Ok(())
    }

    async fn get_usage_records_by_date_range(
        &self,
        date_range: (DateTime, DateTime),
    ) -> anyhow::Result<Vec<UsageRecord>> {
        let collection = self.collection::<UsageRecord>(USAGE_RECORD_COLLECTION_NAME);
        let filter =
            bson::doc! { "usage_date": doc! { GTE_OP: date_range.0, LTE_OP: date_range.1 } };
        let cursor = collection.find(filter).await?;
        let records = cursor.try_collect().await?;
        Ok(records)
    }
    async fn fold_usage_records_by_date_range<T, F>(
        &self,
        date_range: (DateTime, DateTime),
        init: T,
        mut f: F,
    ) -> anyhow::Result<T>
    where
        T: Send + Sync,
        F: Send + Sync + FnMut(T, UsageRecord) -> T,
    {
        let collection = self.collection::<UsageRecord>(USAGE_RECORD_COLLECTION_NAME);
        let filter =
            bson::doc! { "usage_date": doc! { GTE_OP: date_range.0, LTE_OP: date_range.1 } };
        let cursor = collection.find(filter).await?;
        let result = cursor
            .try_fold(init, |acc, record| {
                futures::future::ready(Ok(f(acc, record)))
            })
            .await?;
        Ok(result)
    }
}
