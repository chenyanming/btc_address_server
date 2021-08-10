use actix_web::{
    delete, error, get, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
    Result,
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
#[get("/user/{id}")]
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

// Handler for POST /user
#[post("/user")]
pub async fn add_user(
    pool: web::Data<Pool>,
    item: web::Json<InputUser>,
) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");
    let user = web::block(move || add_single_user(item, &conn))
        .await
        .map_err(|_| HttpResponse::InternalServerError())?;
    match user {
        Ok(user) => Ok(HttpResponse::Created().json(user)),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}

fn add_single_user(
    item: web::Json<InputUser>,
    conn: &PgConnection,
) -> Result<models::User, diesel::result::Error> {
    use crate::schema::users::dsl::*;
    let new_user = models::NewUser {
        first_name: &item.first_name,
        last_name: &item.last_name,
        email: &item.email,
        created_at: chrono::Local::now().naive_local(),
    };
    let res = insert_into(users).values(&new_user).get_result(conn)?;
    Ok(res)
}

// Handler for DELETE /user/{id}
#[delete("/users/{id}")]
pub async fn delete_user(pool: web::Data<Pool>, id: web::Path<u32>) -> Result<HttpResponse, Error> {
    let id = id.into_inner();
    let conn = pool.get().expect("couldn't get db connection from pool");
    let user = web::block(move || delete_single_user(id, &conn))
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

fn delete_single_user(id: u32, conn: &PgConnection) -> Result<usize, diesel::result::Error> {
    use crate::schema::users::dsl::*;
    let count = delete(users.find(id)).execute(conn)?;
    Ok(count)
}
