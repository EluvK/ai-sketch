use anyhow::Ok;
use bson::{DateTime, doc};
use chrono::NaiveDate;
use salvo::Scribe;
use serde::{Deserialize, Serialize};

use super::{
    Statistic,
    common::{StatisticContent, StatisticsType},
};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct OverviewStatisticResponse {
    pub date: NaiveDate,
    pub r#type: StatisticsType,
    pub update_time: DateTime,
    pub user_summary: i64,
    pub user_active_weekly: i64,
    pub token_usage_summary: i64,
    pub token_usage_weekly: i64,
    pub amount_summary: i64,
    pub amount_weekly: i64,
}

impl Scribe for OverviewStatisticResponse {
    fn render(self, res: &mut salvo::Response) {
        res.render(salvo::writing::Json(&self));
    }
}

impl TryFrom<Statistic> for OverviewStatisticResponse {
    type Error = anyhow::Error;

    fn try_from(statistic: Statistic) -> Result<Self, Self::Error> {
        if let StatisticContent::Overview(overview_statistic) = statistic.content {
            Ok(OverviewStatisticResponse {
                date: statistic.date,
                r#type: statistic.r#type,
                update_time: statistic.update_time,
                user_summary: overview_statistic.user_summary,
                user_active_weekly: overview_statistic.user_active_weekly,
                token_usage_summary: overview_statistic.token_usage_summary,
                token_usage_weekly: overview_statistic.token_usage_weekly,
                amount_summary: overview_statistic.amount_summary,
                amount_weekly: overview_statistic.amount_weekly,
            })
        } else {
            Err(anyhow::anyhow!("Invalid statistic content type"))
        }
    }
}
