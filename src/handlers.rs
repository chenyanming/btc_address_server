use actix_web::{
    error, get, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};

use crate::http::Pool;

use crate::models;

use diesel::dsl::{delete, insert_into};
use diesel::prelude::*;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct InputUser {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

// // Handler for GET /users
// pub async fn get_users(db: web::Data<Pool>) -> Result<HttpResponse, actix_web::Error> {
//     let users = web::block(move || get_all_users(db)) // turn function to Future
//         .await
//         .map(|user| {HttpResponse::Ok().json(user).finish()}) // turn every user into json
//         .map_err(|_| HttpResponse::InternalServerError().finish())?;

//     Ok(users)
// }

// fn get_all_users(pool: web::Data<Pool>) -> Result<Vec<User>, diesel::result::Error> {
//     let conn = pool.get().unwrap();
//     let items = users.load::<User>(&conn)?;
//     Ok(items)
// }

/// Finds user by UID.
#[get("/user/{user_id}")]
async fn get_user_by_id(
    pool: web::Data<Pool>,
    id: web::Path<u32>,
) -> std::result::Result<HttpResponse, Error> {
    let id = id.into_inner();
    let conn = pool.get().expect("couldn't get db connection from pool");

    // use web::block to offload blocking Diesel code without blocking server thread
    let user = web::block(move || find_user_by_id(id, &conn))
        .await
        .map_err(|_| HttpResponse::InternalServerError().finish())?;

    match user {
        Ok(user) => Ok(HttpResponse::Ok().json(user)),
        Err(_) => {
            let res = HttpResponse::NotFound().body(format!("No user found with id: {}", id));
            Ok(res)
        }
    }
}
/// Run query using Diesel to find user by uid and return it.
pub fn find_user_by_id(
    id: u32,
    conn: &PgConnection,
) -> Result<models::User, diesel::result::Error> {
    use crate::schema::users::dsl::*;
    users.find(id).get_result::<models::User>(conn)
}

// // Handler for POST /users
// #[post("/users")]
// pub async fn add_user(
//     db: web::Data<Pool>,
//     item: web::Json<InputUser>,
// ) -> Result<HttpResponse, Error> {
//     Ok(web::block(move || add_single_user(db, item))
//         .await
//         .map(|user| HttpResponse::Created().json(user))
//         .map_err(|_| HttpResponse::InternalServerError())?)
// }

// // Handler for DELETE /users/{id}
// #[delete("/users/{id}")]
// pub async fn delete_user(
//     db: web::Data<Pool>,
//     user_id: web::Path<i32>,
// ) -> Result<HttpResponse, Error> {
//     Ok(
//         web::block(move || delete_single_user(db, user_id.into_inner()))
//             .await
//             .map(|user| HttpResponse::Ok().json(user))
//             .map_err(|_| HttpResponse::InternalServerError())?,
//     )
// }

// fn db_get_user_by_id(pool: web::Data<Pool>, user_id: i32) -> Result<User, diesel::result::Error> {
//     let conn = pool.get().unwrap();
//     users.find(user_id).get_result::<User>(&conn)
// }

// fn add_single_user(
//     db: web::Data<Pool>,
//     item: web::Json<InputUser>,
// ) -> Result<User, diesel::result::Error> {
//     let conn = db.get().unwrap();
//     let new_user = NewUser {
//         first_name: &item.first_name,
//         last_name: &item.last_name,
//         email: &item.email,
//         created_at: chrono::Local::now().naive_local(),
//     };
//     let res = insert_into(users).values(&new_user).get_result(&conn)?;
//     Ok(res)
// }

// fn delete_single_user(db: web::Data<Pool>, user_id: i32) -> Result<usize, diesel::result::Error> {
//     let conn = db.get().unwrap();
//     let count = delete(users.find(user_id)).execute(&conn)?;
//     Ok(count)
// }
