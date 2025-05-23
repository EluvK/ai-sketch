use ai_flow_synth::utils::MongoClient;
use async_trait::async_trait;
use bson::{DateTime, doc};
use futures_util::TryStreamExt;
use mongodb::{IndexModel, options::UpdateOptions};
use serde::{Deserialize, Serialize};

use crate::model::constant::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsageRecord {
    pub user_id: String,
    pub provider: String,
    pub llm_model: String,
    pub token_cost: f64, //?why f64
    pub usage_date: DateTime,
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
    async fn save_usage_record(&self, record: UsageRecord) -> anyhow::Result<Option<UsageRecord>>;
    async fn get_usage_records_by_user_id(
        &self,
        record: UsageRecord,
        date_range: Option<(DateTime, DateTime)>,
    ) -> anyhow::Result<Vec<UsageRecord>>;
    async fn get_usage_records_by_date_range(
        &self,
        date_range: (DateTime, DateTime),
    ) -> anyhow::Result<Vec<UsageRecord>>;
}

#[async_trait]
impl UsageRecordRepository for MongoClient {
    async fn save_usage_record(&self, record: UsageRecord) -> anyhow::Result<Option<UsageRecord>> {
        let collection = self.collection::<UsageRecord>(USAGE_RECORD_COLLECTION_NAME);
        let filter = bson::doc! { "user_id": record.user_id.clone(), "provider": record.provider.clone(), "llm_model": record.llm_model.clone(), "usage_date": record.usage_date };
        let update = bson::doc! { SET_OP: { "token_cost": record.token_cost } };
        let options = UpdateOptions::builder().upsert(true).build();
        collection
            .update_one(filter, update)
            .with_options(options)
            .await?;
        Ok(Some(record))
    }
    async fn get_usage_records_by_user_id(
        &self,
        record: UsageRecord,
        date_range: Option<(DateTime, DateTime)>,
    ) -> anyhow::Result<Vec<UsageRecord>> {
        let collection = self.collection::<UsageRecord>(USAGE_RECORD_COLLECTION_NAME);
        let mut filter = bson::doc! { "user_id": record.user_id.clone(), "provider": record.provider.clone(), "llm_model": record.llm_model.clone() };
        if let Some((start_date, end_date)) = date_range {
            filter.insert("usage_date", doc! { GTE_OP: start_date, LTE_OP: end_date });
        }
        let cursor = collection.find(filter).await?;
        let records: Vec<UsageRecord> = cursor.try_collect().await?;
        Ok(records)
    }
    async fn get_usage_records_by_date_range(
        &self,
        date_range: (DateTime, DateTime),
    ) -> anyhow::Result<Vec<UsageRecord>> {
        let collection = self.collection::<UsageRecord>(USAGE_RECORD_COLLECTION_NAME);
        let filter =
            bson::doc! { "usage_date": doc! { GTE_OP: date_range.0, LTE_OP: date_range.1 } };
        let cursor = collection.find(filter).await?;
        let records: Vec<UsageRecord> = cursor.try_collect().await?;
        Ok(records)
    }
}
