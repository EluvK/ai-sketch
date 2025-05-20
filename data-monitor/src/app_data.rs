use ai_flow_synth::utils::MongoClient;

use crate::{config::Config, model::create_all_index};

#[derive(Debug)]
pub struct AppData {
    pub mongo_client: MongoClient,
}

pub type AppDataRef = std::sync::Arc<AppData>;

impl AppData {
    pub async fn new(config: &Config) -> AppDataRef {
        let mongo_client = MongoClient::new(&config.mongo_config)
            .await
            .expect("Failed to create MongoDB client");

        create_all_index(&mongo_client)
            .await
            .expect("Failed to create indexes");

        let data = AppData { mongo_client };
        std::sync::Arc::new(data)
    }
}
