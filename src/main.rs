use actix_web::{delete, get, post, put, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::sync::Arc;
use uuid::Uuid;

// User data structure
#[derive(Debug, Serialize, Deserialize, Clone)]
struct User {
    id: String,
    name: String,
    email: String,
}

// Request structure for creating users (without ID)
#[derive(Debug, Serialize, Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

// Application state holding our database
struct AppState {
    db: Arc<Db>,
}

#[post("/users")]
async fn create_user(
    data: web::Data<AppState>,
    user_data: web::Json<CreateUserRequest>,
) -> impl Responder {
    let id = Uuid::new_v4().to_string();
    let user = User {
        id: id.clone(),
        name: user_data.name.clone(),
        email: user_data.email.clone(),
    };

    match data.db.insert(&id, serde_json::to_vec(&user).unwrap()) {
        Ok(_) => HttpResponse::Created().json(user),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/users/{id}")]
async fn get_user(data: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();

    match data.db.get(&id) {
        Ok(Some(user)) => {
            let user: User = serde_json::from_slice(&user).unwrap();
            HttpResponse::Ok().json(user)
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[put("/users/{id}")]
async fn update_user(
    data: web::Data<AppState>,
    path: web::Path<String>,
    user_data: web::Json<CreateUserRequest>,
) -> impl Responder {
    let id = path.into_inner();
    let user = User {
        id: id.clone(),
        name: user_data.name.clone(),
        email: user_data.email.clone(),
    };

    match data.db.insert(&id, serde_json::to_vec(&user).unwrap()) {
        Ok(_) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[delete("/users/{id}")]
async fn delete_user(data: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();

    match data.db.remove(&id) {
        Ok(Some(_)) => HttpResponse::NoContent().finish(),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Open Sled database
    let db = sled::open("my_db").expect("Failed to open database");
    let db = Arc::new(db);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState { db: Arc::clone(&db) }))
            .service(create_user)
            .service(get_user)
            .service(update_user)
            .service(delete_user)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}