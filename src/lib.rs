use anyhow::Context;
use axum::{
    routing::{delete, get, post, put, Router},
    Extension,
};
use bb8_diesel::DieselConnectionManager;
use diesel::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod util;

mod error;
mod event;
mod schema;
mod sqlite_mapping;
mod user;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

// `derive(OpenApi)` automatically generates code for us which allows us to proved it as a
// parameter to the `SwaggerUi` later.
//
// All paths and types exposed by the API have to be listed here or they won't show up in Swagger.
#[derive(OpenApi)]
#[openapi(
    paths(
        user::get_all,
        user::get_by_username,
        user::post,
        event::get_all,
        event::get_by_id,
        event::post,
        event::delete_by_id,
        event::put,
    ),
    components(schemas(
        user::User,
        user::PostUser,
        event::Event,
        event::PostEvent,
        event::PutEvent
    ))
)]
struct ApiDoc;

// This is where all of the routing happens.
pub async fn api_route(pool: SqlitePool) -> anyhow::Result<Router> {
    Ok(Router::new()
        // SwaggerUi will create its paths under /swagger.
        // The ApiDoc::openapi() function was generated by the derive on ApiDoc.
        .merge(SwaggerUi::new("/swagger").url("/api-doc/openapi.json", ApiDoc::openapi()))
        // Routes defined by this application, first we have the path, then the function which
        // handles requests for that path wrapped by a function with the name of the http method
        // that should be listened for.
        .route("/api/user", get(user::get_all))
        .route("/api/user/:username", get(user::get_by_username))
        .route("/api/user", post(user::post))
        .route("/api/event", get(event::get_all))
        .route("/api/event", post(event::post))
        .route("/api/event/:id", get(event::get_by_id))
        .route("/api/event/:id", delete(event::delete_by_id))
        .route("/api/event/:id", put(event::put))
        .layer(Extension(pool)))
}

// This just renames the type to make it shorter to type.
type SqlitePool = bb8::Pool<DieselConnectionManager<SqliteConnection>>;

// Database Initialization
pub async fn setup_database(database_url: String) -> anyhow::Result<SqlitePool> {
    let manager = DieselConnectionManager::<SqliteConnection>::new(database_url);

    let pool = bb8::Pool::builder()
        .build(manager)
        .await
        .context("Failed to build sqlite pool")?;

    let cpool = pool.clone();
    let mut connection = cpool.get().await.expect("can connect to sqlite");
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("migrations should be tested to work without error");

    Ok(pool)
}
