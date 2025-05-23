use ai_flow_synth::utils::MongoClient;
use bson::doc;
use chrono::NaiveDate;

use crate::{
    model::{constant::*, *},
    utils::date_to_bson_range,
};

pub async fn calculate_overview_statistics(
    mongo_client: &MongoClient,
    date: &NaiveDate,
) -> anyhow::Result<Statistic> {
    let st7 = date_to_bson_range(&(*date - chrono::Duration::days(7)))?.0;
    let st30 = date_to_bson_range(&(*date - chrono::Duration::days(30)))?.1;
    let ed = date_to_bson_range(date)?.1;

    // user
    // ? consider add more indexes here for performance
    let user_collection = mongo_client.collection::<User>(USER_COLLECTION_NAME);
    let user_summary = user_collection
        .count_documents(doc! { "created_at": { LTE_OP: ed } })
        .await? as i64;
    let user_active_weekly = user_collection
        .count_documents(doc! {
            "last_login_at": {
                GTE_OP: st7,
                LTE_OP: ed
            }
        })
        .await? as i64;

    // token usage
    let token_usage_weekly = mongo_client
        .get_usage_records_by_date_range((st7, ed))
        .await?
        .into_iter()
        .fold(0, |acc, record| acc + record.token_cost);
    let token_usage_monthly = mongo_client
        .get_usage_records_by_date_range((st30, ed))
        .await?
        .into_iter()
        .fold(0, |acc, record| acc + record.token_cost);

    // todo
    let amount_monthly = 0;
    let amount_weekly = 0;

    Ok(Statistic {
        date: date.to_owned(),
        r#type: StatisticsType::Overview,
        content: StatisticContent::Overview(OverviewStatistic {
            user_active_weekly,
            user_summary,
            token_usage_weekly,
            token_usage_monthly,
            amount_weekly,
            amount_monthly,
        }),
        update_time: bson::DateTime::now(),
    })
}
