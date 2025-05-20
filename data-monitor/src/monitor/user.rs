use crate::model::{DailyStatistics, DailyStatisticsType, User, constant::*};

use ai_flow_synth::utils::MongoClient;
use bson::{DateTime, doc};
use chrono::{NaiveDate, Utc};

/// 统计指定日期的用户数据
pub async fn calculate_user_statistics(
    client: &MongoClient,
    date_str: &str,
) -> anyhow::Result<DailyStatistics> {
    let collection = client.collection::<User>(USER_COLLECTION_NAME);

    let (start, end) = date_string_to_bson_range(date_str)?;

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

    Ok(DailyStatistics {
        date: date_str.to_owned(),
        r#type: DailyStatisticsType::UserNumbers,
        increment: new_users as i64,
        total: total_users as i64,
        active: active_users as i64,
    })
}

/// 从日期字符串 ("YYYY-MM-DD") 转换为 BSON DateTime 的起止时间
fn date_string_to_bson_range(date_str: &str) -> anyhow::Result<(DateTime, DateTime)> {
    // 解析日期字符串为 NaiveDate
    let naive_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|e| anyhow::anyhow!("Invalid date format: {}", e))?;

    // 计算起止时间
    let start_of_day = naive_date
        .and_hms_opt(0, 0, 0)
        .ok_or(anyhow::anyhow!("Invalid start_of_day"))?
        .and_local_timezone(Utc)
        .single()
        .ok_or(anyhow::anyhow!(
            "Ambiguous or nonexistent start_of_day timezone"
        ))?;

    let end_of_day = naive_date
        .and_hms_opt(23, 59, 59)
        .ok_or(anyhow::anyhow!("Invalid end_of_day"))?
        .and_local_timezone(Utc)
        .single()
        .ok_or(anyhow::anyhow!(
            "Ambiguous or nonexistent end_of_day timezone"
        ))?;

    // 转换为 BSON DateTime
    let bson_start = DateTime::from_millis(start_of_day.timestamp_millis());
    let bson_end = DateTime::from_millis(end_of_day.timestamp_millis());

    Ok((bson_start, bson_end))
}

/// 从 BSON DateTime 转换回日期字符串 ("YYYY-MM-DD")
fn bson_to_date_string(bson_datetime: DateTime) -> String {
    let chrono_datetime = bson_datetime.to_chrono();
    chrono_datetime.date_naive().to_string()
}
