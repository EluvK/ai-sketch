use ai_flow_synth::utils::MongoClient;
use bson::doc;
use futures::TryStreamExt;
use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};

use crate::{error::ServiceResult, model::constant::*};

pub mod schema {
    use salvo::{
        Response, Scribe,
        oapi::{ToResponse, ToSchema},
        writing::Json,
    };
    use serde::{Deserialize, Serialize};

    use crate::model::folder::{Folder, FolderType};

    #[derive(Debug, Serialize, Deserialize, ToSchema)]
    #[serde(rename_all = "camelCase")]
    pub struct FolderResponse {
        pub id: String,
        pub parent_id: Option<String>,

        pub name: String,
        pub description: Option<String>,
        pub r#type: FolderType,
    }

    #[derive(Debug, Serialize, Deserialize, ToResponse, ToSchema)]
    pub struct ListFoldersResponse(pub Vec<FolderResponse>);

    impl Scribe for ListFoldersResponse {
        fn render(self, res: &mut Response) {
            res.render(Json(self));
        }
    }

    impl From<Folder> for FolderResponse {
        fn from(folder: Folder) -> Self {
            FolderResponse {
                id: folder.id,
                parent_id: folder.parent_id,

                name: folder.name,
                description: folder.description,
                r#type: folder.r#type,
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    #[serde(rename = "_id")]
    pub id: String, // uuid
    pub parent_id: Option<String>, // uuid of parent folder
    pub user_id: String,           // uuid of the user who owns this folder
    pub created_at: bson::DateTime,
    pub updated_at: bson::DateTime,

    pub name: String,
    pub description: Option<String>,
    pub r#type: FolderType,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum FolderType {
    #[serde(rename = "system")]
    SystemDefined,
    #[serde(rename = "user")]
    UserDefined,
}

pub async fn create_index(client: &MongoClient) -> ServiceResult<()> {
    let collection = client.collection::<Folder>(FOLDER_COLLECTION_NAME);
    let index = mongodb::IndexModel::builder()
        .keys(doc! { "user_id": 1, "name": 1 })
        .build();
    collection.create_index(index).await?;
    Ok(())
}

#[async_trait::async_trait]
pub trait FolderRepository: Send + Sync {
    async fn create_folder(&self, folder: Folder) -> ServiceResult<()>;
    async fn get_folder_by_id(&self, id: &str) -> ServiceResult<Option<Folder>>;
    async fn get_folders_by_user_id(&self, user_id: &str) -> ServiceResult<Vec<Folder>>;
    async fn update_folder(&self, folder: Folder) -> ServiceResult<Folder>;
    async fn delete_folder(&self, id: &str) -> ServiceResult<()>;
}

#[async_trait::async_trait]
impl FolderRepository for MongoClient {
    async fn create_folder(&self, folder: Folder) -> ServiceResult<()> {
        self.collection::<Folder>(FOLDER_COLLECTION_NAME)
            .insert_one(folder.clone())
            .await?;
        Ok(())
    }

    async fn get_folder_by_id(&self, id: &str) -> ServiceResult<Option<Folder>> {
        let filter = doc! { "_id": id };
        let result = self
            .collection::<Folder>(FOLDER_COLLECTION_NAME)
            .find_one(filter)
            .await?;
        Ok(result)
    }

    async fn get_folders_by_user_id(&self, user_id: &str) -> ServiceResult<Vec<Folder>> {
        let filter = doc! { "user_id": user_id };
        let cursor = self
            .collection::<Folder>(FOLDER_COLLECTION_NAME)
            .find(filter)
            .await?;
        let folders = cursor.try_collect().await?;
        Ok(folders)
    }

    async fn update_folder(&self, folder: Folder) -> ServiceResult<Folder> {
        let filter = doc! { "_id": &folder.id };
        let update = doc! {
            SET_OP: bson::to_bson(&folder)?,
        };
        self.collection::<Folder>(FOLDER_COLLECTION_NAME)
            .update_one(filter, update)
            .await?;
        Ok(folder)
    }

    async fn delete_folder(&self, id: &str) -> ServiceResult<()> {
        let filter = doc! { "_id": id };
        self.collection::<Folder>(FOLDER_COLLECTION_NAME)
            .delete_one(filter)
            .await?;
        Ok(())
    }
}
