use crate::{
    model::{DailyStatistic, DailyStatisticsType, User, constant::*},
    utils::date_to_bson_range,
};

use ai_flow_synth::utils::MongoClient;
use bson::{DateTime, doc};
use chrono::NaiveDate;

/// 统计指定日期的用户数据
pub async fn calculate_user_statistics(
    client: &MongoClient,
    date: &NaiveDate,
) -> anyhow::Result<DailyStatistic> {
    let collection = client.collection::<User>(USER_COLLECTION_NAME);

    let (start, end) = date_to_bson_range(&date)?;

    // 新增用户数
    let new_users = collection
        .count_documents(doc! {
            "created_at": { "$gte": &start, "$lt": &end }
        })
        .await?;
    // 总用户数
    let total_users = collection
        .count_documents(doc! { "created_at": { "$lt": &end } })
        .await?;

    // 活跃用户数（当天登录过的用户）
    let active_users = collection
        .count_documents(doc! {
            "last_login_at": { "$gte": &start, "$lt": &end }
        })
        .await?;

    Ok(DailyStatistic {
        date: date.to_owned(),
        r#type: DailyStatisticsType::UserNumbers,
        increment: new_users as i64,
        total: total_users as i64,
        active: active_users as i64,
        time: DateTime::now(),
    })
}
