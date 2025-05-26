use crate::{
    model::{DailyStatistic, Statistic, StatisticContent, StatisticsType, User, constant::*},
    utils::date_to_bson_range,
};

use ai_flow_synth::utils::MongoClient;
use bson::{DateTime, doc};
use chrono::NaiveDate;

/// 统计指定日期的用户数据
pub async fn calculate_user_statistics(
    client: &MongoClient,
    date: &NaiveDate,
) -> anyhow::Result<Statistic> {
    let collection = client.collection::<User>(USER_COLLECTION_NAME);

    let (start, end) = date_to_bson_range(&date)?;

    // 新增用户数
    let new_users = collection
        .count_documents(doc! {
            "created_at": { GTE_OP: &start, LTE_OP: &end }
        })
        .await?;
    // 总用户数
    let total_users = collection
        .count_documents(doc! { "created_at": { LTE_OP: &end } })
        .await?;

    // 活跃用户数（当天登录过的用户）
    let active_users = collection
        .count_documents(doc! {
            "last_login": { GTE_OP: &start, LTE_OP: &end }
        })
        .await?;

    Ok(Statistic {
        date: date.to_owned(),
        r#type: StatisticsType::DailyUserNumbers,
        content: StatisticContent::DailyNumbers(DailyStatistic {
            increment: new_users as i64,
            total: total_users as i64,
            active: active_users as i64,
        }),
        update_time: DateTime::now(),
    })
}
