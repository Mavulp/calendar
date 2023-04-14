use axum::extract::rejection::JsonRejection;
use diesel::prelude::*;

use anyhow::Context;
use axum::{extract::Path, Extension, Json};
use diesel::dsl::sql;
use diesel::sql_types::Bool;
use serde::{Deserialize, Serialize};
use tracing::debug;
use ts_rs::TS;
use utoipa::ToSchema;

use std::time::SystemTime;

use crate::error::Error;
use crate::schema::users;
use crate::SqlitePool;

// `derive` automatically generates code for a type. Here we use the following:
//
// Debug: Adds debug formatting which allows printing the type for debugging purposes.
//
// Serialize: This is provided by the serde library and allows serializing into various formats, we
// use it to convert to JSON. The `rename_all` attribute below tells it to change all fields to be
// camelCase instead of the standard snake_case in Rust.
//
// TS: This generates tests which when run via `cargo test` write the bindings to the path given by
// `export_to` attribute.
//
// ToSchema: This generates code neccessary to make Swagger understand it. The `example` field
// attributes show up in Swagger examples. Remember to add these ToSchema types to the list of
// components in `lib.rs`.
//
// Queryable: This allow us to use this type when loading data from the database table with the
// same name as the type.
//
// Insertable: This allow us to use this type when inserting data from the database table with the
// same name as the type.
//
// A comment with three slashes is a doc comment and is turned into documentation, some derives
// make use of that for their own purposes, in this case ToSchema makes it show up in Swagger.
//
/// The definition of a user.
#[derive(Debug, Serialize, TS, ToSchema, Queryable, Insertable)]
#[ts(export, export_to = "dist")]
#[serde(rename_all = "camelCase")]
pub struct User {
    /// The unique username of a user.
    #[schema(example = "alice")]
    pub username: String,

    /// A unix timestamp of when this alias was created.
    #[schema(example = 1670802822)]
    pub created_at: i64,
}

// Here we use an attribute like macro to provide some information needed by Swagger.
// https://docs.rs/utoipa/latest/utoipa/attr.path.html
/// Get a list of all users.
#[utoipa::path(
    get,
    path = "/api/user",
    responses(
        (status = 200, description = "Users are returned", body = [User]),
    )
)]
// The web framework we use (axum) allows us to request various parameters which it can fill in for
// us. `Extension`s are use for arbitrary state we want to keep, in this case we are storing the
// pool of database connections in it which we request here and immediatly extract from the
// `Extension` and then place in the `pool` variable.
//
// Some parameters may change how the API works, adding a `Query` parameter would look for Query
// parameters in the request made to the API and if they don't match this route is not used.
//
// The return value let's us see that this function can return an Error which is defined in
// `src/error.rs`. If everything goes well we return a list of `User`s which is wrapped in a `Json`
// type to let axum (the web framework) know that it should be serialized with serde. (See the
// `Serialize` derive on the `User` type.)
pub async fn get_all(Extension(pool): Extension<SqlitePool>) -> Result<Json<Vec<User>>, Error> {
    // We store a pool of connections to the database, I'm not sure how much sense this makes for
    // sqlite but it does make it simpler to work with it in the context of diesel and axum.
    //
    // We can simply get a connection from that pool whenever we need one.
    let mut conn = pool.get().await.expect("can connect to sqlite");

    // This simply logs to the console, there are a few of these for different log levels but they
    // have to be `use`d from tracing (e.g. use tracing::debug).
    // https://docs.rs/tracing/latest/tracing/#macros
    debug!("Loading all users");

    // Here we load the users, the getting started guide of diesel tells us to bring a bunch of
    // things into scope directly via `use crate::schema::users::dsl::*;` but I don't like how easy
    // it is to have clashing variables with the fields that puts into scope. While there may be a
    // better way for now you can find `use crate::schema::users;` at the top of the file and we
    // access the table via `users::dsl::users` instead of simply `users`.
    let users = users::dsl::users
        // This is where the data is actually loaded which is why we have to pass in the
        // connection, I have not bothered to look into why we have to do the weird `&mut *` dance
        // yet but it's probably because the type doesn't match exactly and is converted by doing
        // that.
        // The function knows what the resulting type should be because the compiler looks at the
        // return type of `get_all` and infers which type `users` needs to have. If it was to fail
        // we could help it with the good old turbofish like this `load::<User>(...)`. `load`
        // always returns a Vec (growable array) of the type that its supposed to load.
        //
        // Alternatives to load can be found here:
        // https://docs.rs/diesel/latest/diesel/prelude/trait.RunQueryDsl.html
        .load(&mut *conn)
        // This `context` method is provided by anyhow and will wrap the returned diesel error in
        // an anyhow::Error which is then automatically converted into an internal server error by
        // the question mark. The `Error` type we define implements `From` for anyhow::Errors by
        // putting them into `InternalError`. https://doc.rust-lang.org/stable/std/convert/trait.From.html
        .context("Failed to load users")?;

    // When logging we can also provide additional values we want to log.
    debug!(count = users.len(), "Returning users");

    // As mentioned above wrapping the data in the `Json` type informs axum that we would like it
    // serialized into json via the serde library.
    Ok(Json(users))
}

// See the `get_all` function right above.
/// Get user by username.
#[utoipa::path(
    get,
    path = "/api/user/{username}",
    responses(
        (status = 200, description = "User data is returned", body = User),
        (status = 404, description = "User does not exist"),
    ),
    params(
        ("username" = String, Path, description = "Username of the user to query"),
    )
)]
// See `get_all` for more information but here we have an example of another parameter.
// The `Path` type will automatically pull in any path parameters. There are more ways of using
// this but simply storing it in a `String` is the most simple.
pub async fn get_by_username(
    Path(username): Path<String>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Json<User>, Error> {
    // See `get_all`.
    let mut conn = pool.get().await.expect("can connect to sqlite");

    // If we don't provide a name to variables listed in logs it will fall back to the name of the
    // variable.
    debug!(username, "Trying to find user by name");

    // Same as in `get_all` but we also filter and only return one value.
    let user = users::dsl::users
        // Once again this looks slightly different than the example in the diesel guide because I
        // don't like how they pull everything into scope with their `use` statement.
        //
        // I believe these are the filters which can be used here:
        // https://docs.rs/diesel/latest/diesel/expression_methods/trait.ExpressionMethods.html
        // https://docs.rs/diesel/latest/diesel/prelude/trait.SqliteExpressionMethods.html
        .filter(users::dsl::username.eq(username))
        // First works just like `load` but it does not return a `Vec` and only requests one value
        // even if more would match the filter.
        .first::<User>(&mut *conn)
        // The call to .optional extracts that error and thus turns Result<bool, DieselError> into
        // Result<Option<bool>, DieselError>. We then return any errors and keep just the
        // Option<bool>.
        .optional()
        // The reason we still have a different error type here is because there could be another
        // kind of issue with the database which causes us not to receive data.
        .context("Failed to query user")?;

    debug!(?user, "Found user");

    user.map(|u| Json(u)).ok_or(Error::NotFound)
}

/// The user object required during creation, the missing fields are generated by the back end.
#[derive(Debug, Deserialize, TS, ToSchema)]
#[ts(export, export_to = "dist")]
#[serde(rename_all = "camelCase")]
pub struct PostUser {
    #[schema(example = "alice")]
    pub username: String,
}

// See the `get_all` function at the top of the file.
/// Create a new user.
#[utoipa::path(
    post,
    path = "/api/user",
    request_body = PostUser,
    responses(
        (status = 200, description = "The user was successfully created."),
    )
)]
pub async fn post(
    Extension(pool): Extension<SqlitePool>,
    request: Result<Json<PostUser>, JsonRejection>,
) -> Result<(), Error> {
    // This allows us to have custom error handling instead of the default axum error.
    let Json(request) = request?;

    // See `get_all`.
    let mut conn = pool.get().await.expect("can connect to sqlite");

    // This check if not neccessary to prevent duplicate database entries because the username is
    // the primary key in the database which means it is unique. It is nice to check for this
    // though since otherwise we get a diesel error during the insert which is difficult to work
    // with and we would default to turning it into an internal server error.
    //
    // To ensure that no users are inserted between this check and the actual insertion we should
    // use a transaction but let's skip that for now.
    //
    // Technically this is the same as in `get_by_username` but we don't care about the returned
    // data. Instead we want to know if any data is returned.
    let result = users::dsl::users
        .filter(users::dsl::username.eq(&request.username))
        // We simply return 1 and tell diesel to treat it as a bool to minimize the amount of data
        // returned since we won't be using it.
        .select(sql::<Bool>("1"))
        .first::<bool>(&mut **conn)
        .optional()
        .context("Failed to check for existing users")?;

    // To check what's happening and to make a point let's log the output of that.
    // Since result is an `Option<bool>` and there is no obvious way to convert it to a `String`
    // Rust doesn't provide the normal `Display` trait for conversions to `String`s.
    // Instead we have to use the `Debug` trait which is not intended for users of the application
    // and creates a `String` that looks quite similar to the Rust type it was created from.
    // The `tracing` log library let's us use `Debug` for parameters by prefixing them with a
    // question mark.
    debug!(
        ?result,
        username = request.username,
        "Checked for existing users with the provided name"
    );

    // Just as an example this is how you would print using the standard library only:
    //
    // Here {} is replaced by the variables and `:?` indicates that we want to use `Debug`
    // formatting. By adding an additional `#` we can format it across multiple lines too.
    // Variables can also be used directly within the `{}` since a recent Rust version.
    //
    // There are many more options too: https://doc.rust-lang.org/std/fmt/index.html
    println!(
        "username: {}, result: {:?}, formatted result: {result:#?}",
        request.username, result
    );

    // Very quick way of getting output for debugging, it prints the file, line number and it's
    // content using `variable = {:#?}` formatting.
    dbg!(result);

    // Now we can check if data was returned when we looked for the user, if it was then we can't
    // create another user with that name.
    if result.is_some() {
        return Err(Error::UserExists);
    }

    // Self explanatory I think, we are just getting the seconds since UNIX_EPOCH.
    let created_at = SystemTime::UNIX_EPOCH.elapsed().unwrap().as_secs() as i64;
    debug!(created_at, username = request.username, "Inserting user");

    // Creating the user expected by the data base, this requires that Insertible is implemented
    // for User (check the derive on User which automatically implements it).
    let user = User {
        username: request.username,
        created_at,
    };

    // The actual insertion of the new user into the users table.
    diesel::insert_into(users::table)
        // The values passed in here have to implement the `Insertible` trait which is
        // automatically implemented by the `Insertible` derive.
        .values(&user)
        // `execute()` just runs the query without expecting any results so it either returns an
        // error or nothing.
        .execute(&mut *conn)
        .context("Failed to insert user")?;

    debug!("Inserted user successfully");

    Ok(())
}
