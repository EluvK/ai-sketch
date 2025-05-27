use ai_flow_synth::utils::MongoClient;
use chrono::NaiveDate;

use crate::{
    model::{constant::*, *},
    utils::date_to_bson_range,
};

pub async fn calculate_usage_statistics(
    client: &MongoClient,
    date: &NaiveDate,
) -> anyhow::Result<Statistic> {
    let collection = client.collection::<UsageRecord>(USAGE_RECORD_COLLECTION_NAME);
    let (start, end) = date_to_bson_range(&date)?;

    // 总和有一点意义，但也需要按模型分开？
    // usage平均值毫无意义，需要摊平到成本的平均值

    todo!()
}
