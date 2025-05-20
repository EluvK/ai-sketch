use ai_flow_synth::utils::{IndexModel, MongoClient};
use bson::doc;
use salvo::async_trait;
use serde::{Deserialize, Serialize};

use crate::model::constant::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub created_at: bson::DateTime,
    pub last_login_at: bson::DateTime,
}

#[async_trait]
pub trait UserRepository {
    async fn get_user(&self, id: &str) -> anyhow::Result<Option<User>>;
    async fn create_user(&self, user: User) -> anyhow::Result<()>;
    async fn update_user(&self, user: User) -> anyhow::Result<()>;
    async fn delete_user(&self, id: &str) -> anyhow::Result<()>;
}

pub async fn create_index(client: &MongoClient) -> anyhow::Result<()> {
    let collection = client.collection::<User>(USER_COLLECTION_NAME);
    let index = IndexModel::builder().keys(doc! { "name": 1 }).build();
    collection.create_index(index).await?;
    Ok(())
}

#[async_trait]
impl UserRepository for MongoClient {
    async fn get_user(&self, id: &str) -> anyhow::Result<Option<User>> {
        let collection = self.collection::<User>(USER_COLLECTION_NAME);
        let filter = bson::doc! { "_id": id };
        let user = collection.find_one(filter).await?;
        Ok(user)
    }

    async fn create_user(&self, user: User) -> anyhow::Result<()> {
        let collection = self.collection::<User>(USER_COLLECTION_NAME);
        collection.insert_one(user).await?;
        Ok(())
    }

    async fn update_user(&self, user: User) -> anyhow::Result<()> {
        let collection = self.collection::<User>(USER_COLLECTION_NAME);
        let filter = bson::doc! { "_id": user.id.clone() };
        let update =
            bson::doc! { SET_OP: { "name": user.name, "last_login_at": user.last_login_at } };
        collection.update_one(filter, update).await?;
        Ok(())
    }

    async fn delete_user(&self, id: &str) -> anyhow::Result<()> {
        let collection = self.collection::<User>(USER_COLLECTION_NAME);
        let filter = bson::doc! { "_id": id };
        collection.delete_one(filter).await?;
        Ok(())
    }
}
