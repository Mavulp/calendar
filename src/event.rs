use anyhow::Context;
use axum::extract::Path;
use axum::{Extension, Json};
use diesel::prelude::*;
use serde::Serialize;
use tracing::debug;
use ts_rs::TS;
use utoipa::ToSchema;

use crate::error::Error;
use crate::schema::events;
use crate::SqlitePool;

#[derive(Debug, Serialize, TS, ToSchema, Queryable, Insertable)]
#[ts(export, export_to = "dist")]
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

    debug!(?event, "Found user");
    Ok(Json(event))
}
