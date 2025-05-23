pub mod constant;

mod monitor;
pub use monitor::*;

mod billing;
mod usage;
mod user;

pub use billing::{BillingRecord, BillingRecordRepository, RecordType};
pub use usage::{UsageRecord, UsageRecordRepository};
pub use user::{User, UserRepository};

pub async fn create_all_index(client: &ai_flow_synth::utils::MongoClient) -> anyhow::Result<()> {
    monitor::create_index(client).await?;

    user::create_index(client).await?;
    usage::create_index(client).await?;
    billing::create_index(client).await?;
    Ok(())
}
