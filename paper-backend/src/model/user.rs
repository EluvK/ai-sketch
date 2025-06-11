use ai_flow_synth::utils::MongoClient;
use bson::doc;
use serde::{Deserialize, Serialize};

use crate::{
    error::{ServiceError, ServiceResult},
    model::constant::*,
};

pub mod schema {
    use salvo::{Response, Scribe, macros::Extractible, writing::Json};
    use serde::{Deserialize, Serialize};

    use crate::model::user::User;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct UserInfoResponse {
        pub uid: String,
        pub username: Option<String>,
        pub phone: Option<String>,
        pub email: Option<String>,
        pub wechat_id: Option<String>,
        pub created_at: bson::DateTime,
        pub updated_at: bson::DateTime,
    }

    impl Scribe for UserInfoResponse {
        fn render(self, res: &mut Response) {
            res.render(Json(self));
        }
    }

    impl From<User> for UserInfoResponse {
        fn from(user: User) -> Self {
            UserInfoResponse {
                uid: user.uid,
                username: user.username,
                phone: user.phone,
                email: user.email,
                wechat_id: user.wechat_id,
                created_at: user.created_at,
                updated_at: user.updated_at,
            }
        }
    }

    #[derive(Debug, Deserialize, Extractible)]
    #[salvo(extract(default_source(from = "body")))]
    pub struct UpdateUserInfo {
        pub username: Option<String>,
        pub password: Option<String>,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub uid: String, // uuid

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

impl User {
    pub fn new_by_phone(phone: String) -> Self {
        let now = bson::DateTime::now();
        User {
            uid: uuid::Uuid::new_v4().to_string(),
            username: None,
            password_hash: None,
            phone: Some(phone),
            phone_hash: None, // todo hash phone
            wechat_id: None,
            email: None,
            created_at: now,
            updated_at: now,
            last_login: None,
        }
    }
}

#[async_trait::async_trait]
pub trait UserRepository {
    async fn create_user(&self, user: User) -> ServiceResult<()>;
    async fn update_user(&self, user: User) -> ServiceResult<()>;
    async fn delete_user(&self, id: String) -> ServiceResult<()>;

    async fn get_user_by_phone(&self, phone: &str) -> ServiceResult<Option<User>>;
    async fn get_user_by_uid(&self, uid: &str) -> ServiceResult<Option<User>>;

    async fn check_non_duplicate(&self, phone: Option<String>) -> ServiceResult<()>;
}

#[async_trait::async_trait]
impl UserRepository for MongoClient {
    async fn create_user(&self, user: User) -> ServiceResult<()> {
        self.collection::<User>(USER_COLLECTION_NAME)
            .insert_one(user)
            .await?;
        Ok(())
    }

    async fn update_user(&self, user: User) -> ServiceResult<()> {
        let filter = doc! { "uid": &user.uid };
        let update = doc! { SET_OP: bson::to_bson(&user)? };
        self.collection::<User>(USER_COLLECTION_NAME)
            .update_one(filter, update)
            .await?;
        Ok(())
    }

    async fn delete_user(&self, uid: String) -> ServiceResult<()> {
        let filter = doc! { "uid": uid };
        self.collection::<User>(USER_COLLECTION_NAME)
            .delete_one(filter)
            .await?;
        Ok(())
    }

    async fn get_user_by_phone(&self, phone: &str) -> ServiceResult<Option<User>> {
        let filter = doc! { "phone": phone };
        let user = self
            .collection::<User>(USER_COLLECTION_NAME)
            .find_one(filter)
            .await?;
        Ok(user)
    }

    async fn get_user_by_uid(&self, uid: &str) -> ServiceResult<Option<User>> {
        let filter = doc! { "uid": uid };
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
