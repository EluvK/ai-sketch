use ai_flow_synth::utils::MongoClient;
use salvo::{
    Depot, Response, Router,
    oapi::{RouterExt, endpoint},
};

use crate::{
    app_data::AppDataRef,
    error::ServiceResult,
    model::{
        folder::{Folder, FolderRepository, schema::ListFoldersResponse},
        user::User,
    },
};

pub fn create_router() -> Router {
    Router::new()
        .push(Router::new().get(list_folders))
        .oapi_tag("folder")
}

#[endpoint(
    status_codes(200, 400, 401),
    responses(
        (status_code = 200, body = ListFoldersResponse, description = "List of folders"),
        (status_code = 400, description = "Bad Request: Validation error"),
        (status_code = 401, description = "Unauthorized: User not authenticated")
    )
)]
async fn list_folders(
    depot: &mut Depot,
    // resp: &mut Response,
) -> ServiceResult<ListFoldersResponse> {
    let state = depot.obtain::<AppDataRef>()?;
    let user = depot.obtain::<User>()?;

    let mut folders: Vec<Folder> = state.mongo_client.get_folders_by_user_id(&user.uid).await?;

    if folders.is_empty() {
        ensure_folder_initialized(&state.mongo_client, &user.uid).await?;
        folders = state.mongo_client.get_folders_by_user_id(&user.uid).await?;
    }

    Ok(ListFoldersResponse(
        folders.into_iter().map(Into::into).collect(),
    ))
}

async fn ensure_folder_initialized(mongo_client: &MongoClient, user_id: &str) -> ServiceResult<()> {
    // todo: initialize system defined folders for user at first login
    // maybe move to user creation logic is more elegant
    Ok(())
}
