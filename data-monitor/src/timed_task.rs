use ai_flow_synth::utils::MongoClient;
use chrono::NaiveDate;
use tracing::{error, info};

use crate::{
    app_data::AppDataRef,
    model::StatisticsRepository,
    monitor::{calculate_overview_statistics, calculate_user_statistics},
};

pub async fn register_timed_task(context: AppDataRef) {
    tokio::spawn(async move {
        let interval = tokio::time::Duration::from_secs(30);
        let mut interval = tokio::time::interval(interval);
        loop {
            interval.tick().await;
            info!("Timed task executed");
            // timed_task(context.clone()).await;
            let mongo_client = context.mongo_client.clone();

            let date = chrono::Local::now().naive_local().date();
            info!("Current date: {}", date);

            for i in 0..20 {
                let date = date - chrono::Duration::days(i);
                info!("Calculating user statistics for date: {}", date);
                calc_user(&mongo_client, &date).await;
                info!("Calculating overview statistics for date: {}", date);
                calc_overview(&mongo_client, &date).await;
            }

            calc_user(&mongo_client, &date).await;
            calc_overview(&mongo_client, &date).await;
        }
    });
}

async fn calc_overview(mongo_client: &MongoClient, date: &NaiveDate) {
    match calculate_overview_statistics(mongo_client, date).await {
        Ok(statistics) => {
            info!("Overview statistics: {:?}", statistics);
            _ = mongo_client.upsert(statistics).await;
        }
        Err(e) => {
            error!("Error calculating overview statistics: {}", e);
        }
    }
}

async fn calc_user(mongo_client: &MongoClient, date: &NaiveDate) {
    match calculate_user_statistics(mongo_client, date).await {
        Ok(statistics) => {
            info!("User statistics: {:?}", statistics);
            _ = mongo_client.upsert(statistics).await;
        }
        Err(e) => {
            error!("Error calculating user statistics: {}", e);
        }
    }
}
