use std::vec;

use ai_flow_synth::utils::MongoClient;
use bson::{DateTime, doc, to_bson};
use futures_util::TryStreamExt;
use mongodb::{IndexModel, options::UpdateOptions};
use salvo::Scribe;
use serde::{Deserialize, Serialize};

use crate::{model::constant::*, utils::bson_to_date_string};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DailyStatistic {
    pub date: String,
    pub r#type: DailyStatisticsType,
    pub increment: i64,
    pub total: i64,
    pub active: i64,
    pub time: DateTime,
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

pub async fn create_index(client: &MongoClient) -> anyhow::Result<()> {
    let collection = client.collection::<DailyStatistic>(DAILY_STATISTICS_COLLECTION_NAME);
    let indexes = vec![
        IndexModel::builder().keys(doc! { "date": 1 }).build(),
        IndexModel::builder().keys(doc! { "type": 1 }).build(),
    ];
    collection.create_indexes(indexes).await?;
    Ok(())
}

#[async_trait::async_trait]
pub trait DailyStatisticsRepository {
    async fn upsert(&self, statistics: DailyStatistic) -> anyhow::Result<u64>;
    // async fn get_by_date(&self, date: String) -> anyhow::Result<Vec<DailyStatistic>>;
    async fn get_by_type(
        &self,
        r#type: DailyStatisticsType,
        time: Option<(DateTime, DateTime)>,
    ) -> anyhow::Result<Vec<DailyStatistic>>;
    async fn get_by_date_and_type(
        &self,
        date: String,
        r#type: DailyStatisticsType,
    ) -> anyhow::Result<Option<DailyStatistic>>;
}

#[async_trait::async_trait]
impl DailyStatisticsRepository for MongoClient {
    async fn upsert(&self, statistics: DailyStatistic) -> anyhow::Result<u64> {
        let collection = self.collection::<DailyStatistic>(DAILY_STATISTICS_COLLECTION_NAME);
        let filter = doc! { "date": &statistics.date, "type": to_bson(&statistics.r#type)? };
        let update = doc! {
            SET_OP: {
                "date": &statistics.date,
                "type": to_bson(&statistics.r#type)?,
                "increment": statistics.increment,
                "total": statistics.total,
                "active": statistics.active,
                "time": statistics.time,
            }
        };
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
        time: Option<(DateTime, DateTime)>,
    ) -> anyhow::Result<Vec<DailyStatistic>> {
        let collection = self.collection::<DailyStatistic>(DAILY_STATISTICS_COLLECTION_NAME);
        let mut filter = doc! { "type": to_bson(&r#type)? };
        if let Some((start, end)) = time {
            let st = bson_to_date_string(&start);
            let ed = bson_to_date_string(&end);
            filter.insert("date", doc! { "$gte": st, "$lt": ed });
        } else {
            let now = bson_to_date_string(&DateTime::now());
            filter.insert("date", now);
        }
        let cursor = collection.find(filter).await?;
        let results: Vec<DailyStatistic> = cursor.try_collect().await?;
        Ok(results)
    }

    async fn get_by_date_and_type(
        &self,
        date: String,
        r#type: DailyStatisticsType,
    ) -> anyhow::Result<Option<DailyStatistic>> {
        let collection = self.collection::<DailyStatistic>(DAILY_STATISTICS_COLLECTION_NAME);
        let filter = doc! { "date": date, "type": to_bson(&r#type)? };
        Ok(collection.find_one(filter).await?)
    }
}
