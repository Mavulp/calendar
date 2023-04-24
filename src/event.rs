use crate::util::unix_timestamp;
use anyhow::Context;
use axum::extract::rejection::JsonRejection;
use axum::extract::Path;
use axum::{Extension, Json};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::debug;
use ts_rs::TS;
use utoipa::ToSchema;

use crate::error::Error;
use crate::schema::events;
use crate::SqlitePool;

#[derive(Debug, Serialize, TS, ToSchema, Queryable, Insertable)]
#[ts(export, export_to = "types/")]
#[serde(rename_all = "camelCase")]

pub struct Event {
    #[schema(example = 1)]
    pub id: i64,
    #[schema(example = "Big Mike")]
    pub title: String,
    #[schema(example = "We hike for 7 days in Norwegian plateau.")]
    pub description: Option<String>,
    #[schema(example = "#87d45d")]
    pub color: Option<String>,
    #[schema(example = 1691226000)]
    pub start_date: i64,
    #[schema(example = 1691830800)]
    pub end_date: i64,
    #[schema(example = 60.0520)]
    pub location_lng: Option<f32>,
    #[schema(example = 7.4142)]
    pub location_lat: Option<f32>,
    #[schema(example = 1691830400)]
    pub created_at: i64,
    #[schema(example = 1691830600)]
    pub edited_at: Option<i64>,
}

/// Get a list of all events
#[utoipa::path(
    get,
    path = "/api/event",
    responses(
        (status = 200, description = "Events are returned", body = [Event]),
    )
)]

// Return all events
pub async fn get_all(Extension(pool): Extension<SqlitePool>) -> Result<Json<Vec<Event>>, Error> {
    let mut conn = pool.get().await.expect("can connect to sqlite");
    debug!("Loading all events");
    let events = events::dsl::events
        .load(&mut *conn)
        .context("Failed to load events")?;

    debug!(count = events.len(), "Returning events");
    Ok(Json(events))
}

/// Get an event by its id
#[utoipa::path(
    get,
    path = "/api/event/{id}",
    responses(
        (status = 200, description = "Event data is returned", body = Event),
        (status = 404, description = "Event does not exist"),
    ),
    params(
        ("id" = i64, Path, description = "Identifier of the event"),
    )
)]

// Return event by its id
pub async fn get_by_id(
    Path(id): Path<i64>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<Event>, Error> {
    let mut conn = pool.get().await.expect("can connect to sqlite");
    debug!(id, "Loading event with id");

    let event = events::dsl::events
        .filter(events::dsl::id.eq(id))
        .first::<Event>(&mut *conn)
        .optional()
        .context("Failed to query event")?
        .ok_or(Error::NotFound)?;

    debug!(?event, "Found Event");
    Ok(Json(event))
}

// Post Event
#[derive(Debug, Deserialize, TS, ToSchema, Insertable)]
#[ts(export, export_to = "types/")]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = events)]
pub struct PostEvent {
    #[schema(example = "Big Mike")]
    pub title: String,
    #[schema(example = "We hike for 7 days in Norwegian plateau.")]
    pub description: Option<String>,
    #[schema(example = "#87d45d")]
    pub color: Option<String>,
    #[schema(example = 1691226000)]
    pub start_date: i64,
    #[schema(example = 1691830800)]
    pub end_date: i64,
    #[schema(example = 60.0520)]
    pub location_lng: Option<f32>,
    #[schema(example = 7.4142)]
    pub location_lat: Option<f32>,
    #[serde(skip, default = "unix_timestamp")]
    pub created_at: i64,
    // #[serde(skip)]
    // pub edited_at: i64,
}

/// Post an event
#[utoipa::path(
    post,
    path = "/api/event/{id}",
    responses(
        (status = 200, description = "Posted an event", body = [PostEvent]),
    )
)]

pub async fn post(
    Extension(pool): Extension<SqlitePool>,
    req: Result<Json<PostEvent>, JsonRejection>,
) -> Result<Json<Event>, Error> {
    let Json(req) = req?;
    let mut conn = pool.get().await.expect("can connect to sqlite");

    // Insert into db
    let event = diesel::insert_into(events::table)
        .values(&req)
        .get_result(&mut *conn)
        .context("Failed to insert event")?;

    debug!("Inserted event successfully");

    Ok(Json(event))
}

// Delete event
#[utoipa::path(
    delete,
    path = "/api/event",
    responses(
        (status = 200, description = "Deleted an event"),
    )
)]

pub async fn delete_by_id(
    Path(id): Path<i64>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<(), Error> {
    let mut conn = pool.get().await.expect("can connect to sqlite");
    diesel::delete(events::dsl::events.filter(events::dsl::id.eq(id)))
        .execute(&mut *conn)
        .context("Failed to delete an event")?;

    Ok(())
}

// Put Event
#[derive(Debug, Deserialize, TS, ToSchema, AsChangeset)]
#[ts(export, export_to = "types/")]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = events)]
pub struct PutEvent {
    #[schema(example = "Big Mike")]
    pub title: Option<String>,
    #[schema(example = "We hike for 7 days in Norwegian plateau.")]
    pub description: Option<String>,
    #[schema(example = "#87d45d")]
    pub color: Option<String>,
    #[schema(example = 1691226000)]
    pub start_date: Option<i64>,
    #[schema(example = 1691830800)]
    pub end_date: Option<i64>,
    #[schema(example = 60.0520)]
    pub location_lng: Option<f32>,
    #[schema(example = 7.4142)]
    pub location_lat: Option<f32>,
    #[serde(skip, default = "unix_timestamp")]
    pub edited_at: i64,
}

#[utoipa::path(
    put,
    path = "/api/event/{id}",
    responses(
        (status = 200, description = "Updated an event", body = [Event]),
    )
)]

pub async fn put(
    Path(id): Path<i64>,
    Extension(pool): Extension<SqlitePool>,
    req: Result<Json<PutEvent>, JsonRejection>,
) -> Result<Json<Event>, Error> {
    let Json(req) = req?;
    let mut conn = pool.get().await.expect("can connect to sqlite");

    let event = diesel::update(events::dsl::events.filter(events::dsl::id.eq(id)))
        .set(&req)
        .get_result(&mut *conn)
        .context("Failed to update event")?;

    Ok(Json(event))
}
