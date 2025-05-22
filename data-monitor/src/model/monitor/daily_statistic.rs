use ai_flow_synth::utils::MongoClient;
use bson::{DateTime, doc, to_bson};
use chrono::NaiveDate;
use futures_util::TryStreamExt;
use mongodb::options::UpdateOptions;
use salvo::Scribe;
use serde::{Deserialize, Serialize};

use super::{Statistic, StatisticContent, common::StatisticsType};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OneDayResponse {
    pub date: NaiveDate,
    pub r#type: StatisticsType,
    pub update_time: DateTime, // update time
    pub increment: i64,
    pub total: i64,
    pub active: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DailyStatisticsResponse(pub Vec<OneDayResponse>);

impl From<OneDayResponse> for DailyStatisticsResponse {
    fn from(statistic: OneDayResponse) -> Self {
        DailyStatisticsResponse(vec![statistic])
    }
}

impl Scribe for OneDayResponse {
    fn render(self, res: &mut salvo::Response) {
        res.render(salvo::writing::Json(&self));
    }
}

impl Scribe for DailyStatisticsResponse {
    fn render(self, res: &mut salvo::Response) {
        res.render(salvo::writing::Json(&self));
    }
}

impl TryFrom<Statistic> for OneDayResponse {
    type Error = anyhow::Error;

    fn try_from(statistic: Statistic) -> Result<Self, Self::Error> {
        if let StatisticContent::DailyNumbers(daily_statistic) = statistic.content {
            Ok(OneDayResponse {
                date: statistic.date,
                r#type: statistic.r#type,
                update_time: statistic.update_time,
                increment: daily_statistic.increment,
                total: daily_statistic.total,
                active: daily_statistic.active,
            })
        } else {
            Err(anyhow::anyhow!("Invalid statistic content type"))
        }
    }
}

impl TryFrom<Vec<Statistic>> for DailyStatisticsResponse {
    type Error = anyhow::Error;

    fn try_from(statistics: Vec<Statistic>) -> Result<Self, Self::Error> {
        let mut responses = Vec::new();
        for statistic in statistics {
            if let StatisticContent::DailyNumbers(daily_statistic) = statistic.content {
                responses.push(OneDayResponse {
                    date: statistic.date,
                    r#type: statistic.r#type,
                    update_time: statistic.update_time,
                    increment: daily_statistic.increment,
                    total: daily_statistic.total,
                    active: daily_statistic.active,
                });
            } else {
                return Err(anyhow::anyhow!("Invalid statistic content type"));
            }
        }
        Ok(DailyStatisticsResponse(responses))
    }
}
