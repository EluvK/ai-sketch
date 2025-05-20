use mongodb::{Client, Database, options::ClientOptions};
use serde::Deserialize;
use std::{error::Error, sync::Arc};

pub use mongodb::{IndexModel, error::Error as MongoError, options::IndexOptions};

#[derive(Debug, Deserialize)]
pub struct MongoConfig {
    pub uri: String,
    pub db_name: String,
}

#[derive(Debug, Clone)]
pub struct MongoClient {
    client: Arc<Client>,
    db: Arc<Database>,
}

impl MongoClient {
    pub async fn new(config: &MongoConfig) -> Result<Self, Box<dyn Error>> {
        let options = ClientOptions::parse(&config.uri).await?;
        let client = Client::with_options(options)?;
        let db = client.database(&config.db_name);
        Ok(MongoClient {
            client: Arc::new(client),
            db: Arc::new(db),
        })
    }

    // pub fn db(&self) -> Arc<Database> {
    //     Arc::clone(&self.db)
    // }

    pub fn collection<T>(&self, name: &str) -> mongodb::Collection<T>
    where
        T: serde::de::DeserializeOwned + serde::Serialize + Unpin + Send + Sync,
    {
        self.db.collection::<T>(name)
    }
}

#[cfg(test)]
mod tests {
    use mongodb::bson::{DateTime, doc};
    use serde::Serialize;

    use super::*;

    #[tokio::test]
    async fn test_mongo_client_connection() {
        let config = MongoConfig {
            uri: "mongodb://localhost:27017".to_string(),
            db_name: "paper".to_string(),
        };

        let client = MongoClient::new(&config)
            .await
            .expect("Failed to create client");
        // let db = client.db();
        // Check that the database name matches
        // assert_eq!(db.name(), "paper");

        #[derive(Debug, Deserialize, Serialize)]
        struct User {
            email: String,
            phone: String,
            role: String,
            created_at: DateTime,
        }

        // Try to find one document in the "user" collection
        let collection = client.collection::<User>("users");
        let result = collection.find_one(doc! {}).await;
        assert!(result.is_ok());
        dbg!(result);
    }
}
