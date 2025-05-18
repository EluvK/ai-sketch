use tracing::info;

use crate::app_data::AppDataRef;

pub async fn register_timed_task(context: AppDataRef) {
    tokio::spawn(async move {
        let interval = tokio::time::Duration::from_secs(10);
        let mut interval = tokio::time::interval(interval);
        loop {
            interval.tick().await;
            info!("Timed task executed");
            // timed_task(context.clone()).await;
        }
    });
}
