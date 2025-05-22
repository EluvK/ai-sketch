use ai_flow_synth::utils::MongoClient;
use bson::doc;
use chrono::NaiveDate;

use crate::{
    model::{OverviewStatistic, Statistic, StatisticContent, StatisticsType, User, constant::*},
    utils::date_to_bson_range,
};

pub async fn calculate_overview_statistics(
    mongo_client: &MongoClient,
    date: &NaiveDate,
) -> anyhow::Result<Statistic> {
    let user_collection = mongo_client.collection::<User>(USER_COLLECTION_NAME);

    let date_before_week = *date - chrono::Duration::days(7);
    let st = date_to_bson_range(&date_before_week)?.0;
    let ed = date_to_bson_range(date)?.1;

    let user_summary = user_collection
        .count_documents(doc! { "created_at": { "$lt": ed } })
        .await? as i64;
    let user_active_weekly = user_collection
        .count_documents(doc! {
            "last_login_at": {
                "$gte": st,
                "$lt": ed
            }
        })
        .await? as i64;

    // todo
    let token_usage_summary = 0;
    let token_usage_weekly = 0;
    let amount_summary = 0;
    let amount_weekly = 0;

    Ok(Statistic {
        date: date.to_owned(),
        r#type: StatisticsType::Overview,
        content: StatisticContent::Overview(OverviewStatistic {
            user_summary,
            user_active_weekly,
            token_usage_summary,
            token_usage_weekly,
            amount_summary,
            amount_weekly,
        }),
        update_time: bson::DateTime::now(),
    })
}
