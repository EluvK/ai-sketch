use ai_flow_synth::utils::MongoClient;
use async_trait::async_trait;
use bson::{DateTime, doc, to_bson};
use futures_util::TryStreamExt;
use mongodb::IndexModel;
use serde::{Deserialize, Serialize};

use crate::model::constant::*;

// the consumption record is not very useful here
// main purpose is to record the recharge records
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BillingRecord {
    pub record_id: String,
    pub user_id: String,
    pub record_type: RecordType,
    pub amount: f64,
    pub balance: f64,
    pub created_at: DateTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordType {
    Recharge,
    Consumption, // not very useful here.
}

pub async fn create_index(client: &MongoClient) -> anyhow::Result<()> {
    let collection = client.collection::<BillingRecord>(BILLING_RECORD_COLLECTION_NAME);
    let user_id_index = IndexModel::builder().keys(doc! { "user_id": 1 }).build();
    let record_id_index = IndexModel::builder().keys(doc! { "record_id": 1 }).build();
    let created_at_index = IndexModel::builder()
        .keys(doc! { "user_id": 1, "created_at": 1 })
        .build();
    collection
        .create_indexes(vec![user_id_index, record_id_index, created_at_index])
        .await?;
    Ok(())
}

#[async_trait]
pub trait BillingRecordRepository {
    async fn save_billing_record(
        &self,
        record: BillingRecord,
    ) -> anyhow::Result<Option<BillingRecord>>;
    async fn get_billing_records_by_user_id(
        &self,
        user_id: String,
        date_range: Option<(DateTime, DateTime)>,
    ) -> anyhow::Result<Vec<BillingRecord>>;
    async fn get_billing_records_by_date_range(
        &self,
        date_range: (DateTime, DateTime),
        record_type: Option<RecordType>,
    ) -> anyhow::Result<Vec<BillingRecord>>;
}

#[async_trait]
impl BillingRecordRepository for MongoClient {
    async fn save_billing_record(
        &self,
        record: BillingRecord,
    ) -> anyhow::Result<Option<BillingRecord>> {
        let collection = self.collection::<BillingRecord>(BILLING_RECORD_COLLECTION_NAME);
        let filter = doc! { "record_id": record.record_id.clone() };
        let update = doc! {
        SET_OP: { "user_id": record.user_id.clone(), "record_type": to_bson(&record.record_type)?, "amount": record.amount, "balance": record.balance, "created_at": record.created_at } };
        collection.update_one(filter, update).await?;
        Ok(Some(record))
    }
    async fn get_billing_records_by_user_id(
        &self,
        user_id: String,
        date_range: Option<(DateTime, DateTime)>,
    ) -> anyhow::Result<Vec<BillingRecord>> {
        let collection = self.collection::<BillingRecord>(BILLING_RECORD_COLLECTION_NAME);
        let mut filter = doc! { "user_id": user_id };
        if let Some((start_date, end_date)) = date_range {
            filter.insert("created_at", doc! { GTE_OP: start_date, LTE_OP: end_date });
        }
        let records = collection.find(filter).await?.try_collect().await?;
        Ok(records)
    }
    async fn get_billing_records_by_date_range(
        &self,
        date_range: (DateTime, DateTime),
        record_type: Option<RecordType>,
    ) -> anyhow::Result<Vec<BillingRecord>> {
        let collection = self.collection::<BillingRecord>(BILLING_RECORD_COLLECTION_NAME);
        let mut filter = doc! { "created_at": doc! { GTE_OP: date_range.0, LTE_OP: date_range.1 } };
        if let Some(record_type) = record_type {
            filter.insert("record_type", to_bson(&record_type)?);
        }
        let records = collection.find(filter).await?.try_collect().await?;
        Ok(records)
    }
}
