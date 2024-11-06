use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::error::Error;

use std::net::SocketAddr;

/*
 * 1. client sends resut to intiate multipart upload, responds with upload id
 * 2. client uploads each file chunk with part number (to maintain ordering), the size of part and
 *    md5 hash
 * 3. validate chunks against hash and size - responds with tag(unique id) for the chunk
 * 4. client issues request to complete the upload which contains a list of each chunk number and
 *    teh assiociate chunk tag, api validates no missing chunks and that the chunk numbers match
 *    the chunk tags
 *  5. assemble the or return error
 */

#[derive(Serialize, Debug)]
struct Upload {
    id: u64,
    name: String,
}

#[derive(Deserialize, Debug)]
struct UploadPayload {
    name: String,
}

#[derive(Serialize, Debug)]
struct Chunk {
    tag: u64,
    part: u64,
    hash: String,
    size: u64,
}

#[derive(Deserialize, Debug)]
struct ChunkPayload {
    part: u64,
    hash: String,
    size: u64,
}

/// Initiate multipart file upload
async fn initiate_upload(
    State(pool): State<PgPool>,
    Json(payload): Json<UploadPayload>,
) -> Response {
    let _result = sqlx::query("INSERT INTO uploads (name) VALUES ($1) RETURNING id")
        .bind(&payload.name)
        .fetch_one(&pool)
        .await
        .expect("query failed");
    (
        StatusCode::CREATED,
        Json(Upload {
            id: 0,
            name: payload.name,
        }),
    )
        .into_response()
}

/// Get data regarding the upload
async fn get_upload(Path(id): Path<u64>) -> impl IntoResponse {
    format!("called get upload with {id}")
}

/// Delete the data regarding the upload and any chunks that have been uploaded
async fn delete_upload(Path(id): Path<u64>) -> impl IntoResponse {
    format!("called delete upload with {id}")
}

/// Upload a chunk of the multipart file
async fn import_chunk(Path(id): Path<u64>) -> impl IntoResponse {
    format!("uploading chunk for item with {id}");
}

/// Get the chunk data so far for the multipart file upload
async fn get_chunks(Path(id): Path<u64>) -> impl IntoResponse {
    format!("get chunks for item with {id}");
}

/// Validate and assemble the final output file
async fn complete_upload(Path(id): Path<u64>) -> impl IntoResponse {
    format!("complete import with {id}");
}

struct DbConnection(sqlx::pool::PoolConnection<sqlx::Postgres>);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let db_connection = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432".to_string());
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(3))
        .connect(&db_connection)
        .await?;

    let app = Router::new()
        .route("/v1/import", post(initiate_upload))
        .route("/v1/import/:id", get(get_upload))
        .route("/v1/import/:id", delete(delete_upload))
        .route("/v1/import/:id/chunk", post(import_chunk))
        .route("/v1/import/:id/chunk", get(get_chunks))
        .route("/v1/import/:id/complete", post(complete_upload))
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 6969));
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::debug!("listening on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
