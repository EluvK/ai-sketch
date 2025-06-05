use ai_flow_synth::utils::MongoClient;
use bson::doc;
use serde::{Deserialize, Serialize};

use crate::{
    error::{ServiceError, ServiceResult},
    model::constant::*,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i64, // uuid

    pub username: Option<String>,
    pub password_hash: Option<String>,
    pub phone: Option<String>,
    pub phone_hash: Option<String>,
    pub wechat_id: Option<String>, // 微信id

    pub email: Option<String>,

    pub created_at: bson::DateTime,
    pub updated_at: bson::DateTime,
    pub last_login: Option<bson::DateTime>,
}

#[async_trait::async_trait]
pub trait UserRepository {
    async fn create(&self, user: User) -> ServiceResult<()>;
    async fn update(&self, user: User) -> ServiceResult<User>;
    async fn delete(&self, id: i64) -> ServiceResult<()>;

    async fn get_by_phone(&self, phone: String) -> ServiceResult<Option<User>>;

    async fn check_non_duplicate(&self, phone: Option<String>) -> ServiceResult<()>;
}

#[async_trait::async_trait]
impl UserRepository for MongoClient {
    async fn create(&self, user: User) -> ServiceResult<()> {
        self.collection(USER_COLLECTION_NAME)
            .insert_one(user)
            .await?;
        Ok(())
    }

    async fn update(&self, user: User) -> ServiceResult<User> {
        let filter = doc! { "id": user.id };
        let update = doc! { SET_OP: bson::to_bson(&user)? };
        self.collection::<User>(USER_COLLECTION_NAME)
            .update_one(filter, update)
            .await?;
        Ok(user)
    }

    async fn delete(&self, id: i64) -> ServiceResult<()> {
        let filter = doc! { "id": id };
        self.collection::<User>(USER_COLLECTION_NAME)
            .delete_one(filter)
            .await?;
        Ok(())
    }

    async fn get_by_phone(&self, phone: String) -> ServiceResult<Option<User>> {
        let filter = doc! { "phone": phone };
        let user = self
            .collection::<User>(USER_COLLECTION_NAME)
            .find_one(filter)
            .await?;
        Ok(user)
    }

    async fn check_non_duplicate(&self, phone: Option<String>) -> ServiceResult<()> {
        if let Some(phone) = phone {
            let filter = doc! { "phone": phone };
            let count = self
                .collection::<User>(USER_COLLECTION_NAME)
                .count_documents(filter)
                .await?;
            if count > 0 {
                return Err(ServiceError::DuplicateUser(format!("User already exists")));
            }
        }
        Ok(())
    }
}
