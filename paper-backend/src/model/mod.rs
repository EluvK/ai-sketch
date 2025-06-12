mod constant;
pub mod folder;
pub mod user;

pub async fn create_all_index(client: &ai_flow_synth::utils::MongoClient) -> anyhow::Result<()> {
    folder::create_index(client).await?;
    user::create_index(client).await?;
    Ok(())
}
