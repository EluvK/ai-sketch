use ai_flow_synth::utils::MongoClient;
use bson::{DateTime, doc, to_bson};
use chrono::NaiveDate;
use futures_util::TryStreamExt;
use mongodb::options::UpdateOptions;
use salvo::Scribe;
use serde::{Deserialize, Serialize};

use crate::{model::constant::*, utils::bson_to_date_string};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DailyStatistic {
    pub date: NaiveDate,
    pub r#type: DailyStatisticsType,
    pub increment: i64,
    pub total: i64,
    pub active: i64,
    pub time: DateTime, // update time
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DailyStatistics(pub Vec<DailyStatistic>);

impl From<DailyStatistic> for DailyStatistics {
    fn from(statistic: DailyStatistic) -> Self {
        DailyStatistics(vec![statistic])
    }
}

impl Scribe for DailyStatistic {
    fn render(self, res: &mut salvo::Response) {
        res.render(salvo::writing::Json(&self));
    }
}

impl Scribe for DailyStatistics {
    fn render(self, res: &mut salvo::Response) {
        res.render(salvo::writing::Json(&self));
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum DailyStatisticsType {
    UserNumbers,
}

#[async_trait::async_trait]
pub trait DailyStatisticsRepository {
    async fn upsert(&self, statistics: DailyStatistic) -> anyhow::Result<u64>;
    // async fn get_by_date(&self, date: String) -> anyhow::Result<Vec<DailyStatistic>>;
    async fn get_by_type(
        &self,
        r#type: DailyStatisticsType,
        date_range: (NaiveDate, NaiveDate),
    ) -> anyhow::Result<Vec<DailyStatistic>>;
    async fn get_by_date_and_type(
        &self,
        date: NaiveDate,
        r#type: DailyStatisticsType,
    ) -> anyhow::Result<Option<DailyStatistic>>;
}

#[async_trait::async_trait]
impl DailyStatisticsRepository for MongoClient {
    async fn upsert(&self, statistics: DailyStatistic) -> anyhow::Result<u64> {
        let collection = self.collection::<DailyStatistic>(ADMIN_STATISTICS_COLLECTION_NAME);
        let filter =
            doc! { "date": to_bson(&statistics.date)?, "type": to_bson(&statistics.r#type)? };
        let update = doc! { SET_OP: bson::to_document(&statistics)? };
        Ok(collection
            .update_one(filter, update)
            .with_options(Some(UpdateOptions::builder().upsert(true).build()))
            .await?
            .modified_count)
    }

    // async fn get_by_date(&self, date: String) -> anyhow::Result<Vec<DailyStatistic>> {
    //     let collection = self.collection::<DailyStatistic>(DAILY_STATISTICS_COLLECTION_NAME);
    //     let filter = doc! { "date": date };
    //     let cursor = collection.find(filter).await?;
    //     let results: Vec<DailyStatistic> = cursor.try_collect().await?;
    //     Ok(results)
    // }

    async fn get_by_type(
        &self,
        r#type: DailyStatisticsType,
        date_range: (NaiveDate, NaiveDate),
    ) -> anyhow::Result<Vec<DailyStatistic>> {
        let collection = self.collection::<DailyStatistic>(ADMIN_STATISTICS_COLLECTION_NAME);
        let filter = doc! { "type": to_bson(&r#type)?, "date": { "$gte": to_bson(&date_range.0)?, "$lte": to_bson(&date_range.1)? } };
        // tracing::info!("Querying statistics with filter: {:?}", filter);
        let cursor = collection.find(filter).await?;
        let results: Vec<DailyStatistic> = cursor.try_collect().await?;
        // tracing::info!("Query results: {:?}", results);
        Ok(results)
    }

    async fn get_by_date_and_type(
        &self,
        date: NaiveDate,
        r#type: DailyStatisticsType,
    ) -> anyhow::Result<Option<DailyStatistic>> {
        let collection = self.collection::<DailyStatistic>(ADMIN_STATISTICS_COLLECTION_NAME);
        let filter = doc! { "date": to_bson(&date)?, "type": to_bson(&r#type)? };
        Ok(collection.find_one(filter).await?)
    }
}
