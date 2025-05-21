pub mod constant;
mod monitor;
mod user;

pub use monitor::{
    DailyStatistic, DailyStatistics, DailyStatisticsRepository, DailyStatisticsType,
};
pub use user::{User, UserRepository};

pub async fn create_all_index(client: &ai_flow_synth::utils::MongoClient) -> anyhow::Result<()> {
    user::create_index(client).await?;
    monitor::create_index(client).await?;
    Ok(())
}
