use ai_flow_synth::utils::MongoClient;
use salvo::{
    Depot, Response, Router, Writer,
    oapi::{
        RouterExt, endpoint,
        extract::{JsonBody, PathParam},
    },
};

use crate::{
    app_data::AppDataRef,
    error::{ServiceError, ServiceResult},
    model::{
        folder::{
            Folder, FolderRepository,
            schema::{
                CreateFolderRequest, FolderResponse, ListFoldersResponse, UpdateFolderRequest,
            },
        },
        user::User,
    },
};

pub fn create_router() -> Router {
    Router::new()
        .push(Router::new().get(list_folders).post(create_folder))
        .push(
            Router::with_path("{folder_id}")
                .put(update_folder)
                .push(Router::with_path("literatures").get(get_folder_literatures)),
        )
        .oapi_tag("folder")
}

/// List Folders
///
/// Lists all folders for the authenticated user.
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

/// Create Folder
///
/// Creates a new user-defined folder for the authenticated user.
#[endpoint(
    status_codes(201, 400, 401),
    responses(
        (status_code = 201, body = FolderResponse, description = "Folder created successfully"),
        (status_code = 400, description = "Bad Request: Validation error"),
        (status_code = 401, description = "Unauthorized: User not authenticated")
    )
)]
async fn create_folder(
    depot: &mut Depot,
    request: JsonBody<CreateFolderRequest>,
    resp: &mut Response,
) -> ServiceResult<FolderResponse> {
    let state = depot.obtain::<AppDataRef>()?;
    let user = depot.obtain::<User>()?;

    // Validate the request
    if let Some(parent_id) = request.parent_id.as_ref() {
        if state
            .mongo_client
            .get_folder_by_id(parent_id)
            .await?
            .is_none()
        {
            return Err(ServiceError::BadRequest(
                "Parent folder does not exist".to_string(),
            ));
        }
    }

    let folder = Folder::new_from_request(&user.uid, request.0);
    state.mongo_client.create_folder(folder.clone()).await?;
    resp.status_code(salvo::http::StatusCode::CREATED);
    Ok(folder.into())
}

/// Update Folder
///
/// Updates an existing folder for the authenticated user.
#[endpoint(
    status_codes(200, 400, 401, 404),
    request_body(content = UpdateFolderRequest, description = "Update folder details"),
    responses(
        (status_code = 200, body = FolderResponse, description = "Folder updated successfully"),
        (status_code = 400, description = "Bad Request: Validation error"),
        (status_code = 401, description = "Unauthorized: User not authenticated"),
        (status_code = 404, description = "Not Found: Folder does not exist")
    )
)]
async fn update_folder(
    depot: &mut Depot,
    folder_id: PathParam<String>,
    request: JsonBody<UpdateFolderRequest>,
) -> ServiceResult<FolderResponse> {
    let state = depot.obtain::<AppDataRef>()?;
    let user = depot.obtain::<User>()?;

    // Validate the folder ID
    if folder_id.is_empty() {
        return Err(ServiceError::BadRequest(
            "Folder ID cannot be empty".to_string(),
        ));
    }

    // Fetch the folder by ID
    let mut folder = state
        .mongo_client
        .get_folder_by_id(&folder_id)
        .await?
        .ok_or_else(|| ServiceError::NotFound(format!("Folder with ID {} not found", folder_id)))?;

    // Ensure the folder belongs to the authenticated user
    if folder.user_id != user.uid {
        return Err(ServiceError::Unauthorized(
            "You do not have permission to update this folder".to_string(),
        ));
    }

    // Update the folder details
    if let Some(name) = request.0.name {
        folder.name = name;
    }
    folder.description = request.0.description;
    if let Some(parent_id) = request.0.parent_id.as_ref() {
        if state
            .mongo_client
            .get_folder_by_id(parent_id)
            .await?
            .is_none()
        {
            return Err(ServiceError::BadRequest(
                "Parent folder does not exist".to_string(),
            ));
        }
        folder.parent_id = Some(parent_id.clone());
    }

    let updated_folder = state.mongo_client.update_folder(folder).await?;
    Ok(updated_folder.into())
}

async fn ensure_folder_initialized(mongo_client: &MongoClient, user_id: &str) -> ServiceResult<()> {
    let default_system_folder = Folder::default_system_folder(user_id);
    mongo_client.create_folder(default_system_folder).await?;
    // todo: initialize system defined folders for user at first login
    // maybe move to user creation logic is more elegant
    Ok(())
}

/// Get Folder's Literatures
///
/// Gets the details of a specific folder, including its literatures, for the authenticated user.
#[endpoint(
    status_codes(200, 400, 401, 404),
    responses(
        (status_code = 200, body = FolderResponse, description = "Folder details retrieved successfully"),
        (status_code = 400, description = "Bad Request: Validation error"),
        (status_code = 401, description = "Unauthorized: User not authenticated"),
        (status_code = 404, description = "Not Found: Folder does not exist")
    )
)]
async fn get_folder_literatures(
    depot: &mut Depot,
    folder_id: PathParam<String>,
) -> ServiceResult<FolderResponse> {
    // todo resp should be literature list.
    // todo add param limit and marker for pagination
    let state = depot.obtain::<AppDataRef>()?;
    let user = depot.obtain::<User>()?;

    // Validate the folder ID
    if folder_id.is_empty() {
        return Err(ServiceError::BadRequest(
            "Folder ID cannot be empty".to_string(),
        ));
    }

    // Fetch the folder by ID
    let folder = state
        .mongo_client
        .get_folder_by_id(&folder_id)
        .await?
        .ok_or_else(|| ServiceError::NotFound(format!("Folder with ID {} not found", folder_id)))?;

    // Ensure the folder belongs to the authenticated user
    if folder.user_id != user.uid {
        return Err(ServiceError::Unauthorized(
            "You do not have permission to access this folder".to_string(),
        ));
    }

    Ok(folder.into())
}
