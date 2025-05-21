use ai_flow_synth::utils::MongoClient;
use tracing::{error, info};

use crate::{
    app_data::AppDataRef, model::DailyStatisticsRepository, monitor::calculate_user_statistics,
};

pub async fn register_timed_task(context: AppDataRef) {
    tokio::spawn(async move {
        let interval = tokio::time::Duration::from_secs(10);
        let mut interval = tokio::time::interval(interval);
        loop {
            interval.tick().await;
            info!("Timed task executed");
            // timed_task(context.clone()).await;
            let mongo_client = context.mongo_client.clone();

            let date = chrono::Local::now().format("%Y-%m-%d").to_string();
            info!("Current date: {}", date);

            calc_user(mongo_client, date).await;
        }
    });
}

async fn calc_user(mongo_client: MongoClient, date: String) {
    match calculate_user_statistics(&mongo_client, &date).await {
        Ok(statistics) => {
            info!("User statistics: {:?}", statistics);
            _ = mongo_client.upsert(statistics).await;
        }
        Err(e) => {
            error!("Error calculating user statistics: {}", e);
        }
    }
}
