mod constant;
mod user;

pub use user::{User, UserRepository};

pub async fn create_all_index(client: &ai_flow_synth::utils::MongoClient) -> anyhow::Result<()> {
    user::create_index(client).await?;
    Ok(())
}
