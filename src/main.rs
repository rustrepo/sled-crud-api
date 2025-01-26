use actix_web::{delete, get, post, put, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct User {
    id: String,
    name: String,
    email: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

struct AppState {
    db: Arc<Mutex<Db>>,
}

impl AppState {
    async fn db_operation<F, R>(&self, op: F) -> Result<R, sled::Error>
    where
        F: FnOnce(&Db) -> Result<R, sled::Error> + Send + 'static,
        R: Send + 'static,
    {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let db = db.blocking_lock();
            op(&db)
        })
        .await
        .unwrap()
    }
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

    // Clone values for the closure
    let user_clone = user.clone();
    let id_clone = id.clone();

    match data
        .db_operation(move |db| {
            db.insert(&id_clone, serde_json::to_vec(&user_clone).unwrap())
        })
        .await
    {
        Ok(_) => HttpResponse::Created().json(user),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/users/{id}")]
async fn get_user(data: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    
    // Clone ID for the closure
    let id_clone = id.clone();

    match data
        .db_operation(move |db| db.get(&id_clone))
        .await
    {
        Ok(Some(user)) => match serde_json::from_slice::<User>(&user) {
            Ok(user) => HttpResponse::Ok().json(user),
            Err(_) => HttpResponse::InternalServerError().finish(),
        },
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

    // Clone values for the closure
    let user_clone = user.clone();
    let id_clone = id.clone();

    match data
        .db_operation(move |db| {
            db.insert(&id_clone, serde_json::to_vec(&user_clone).unwrap())
        })
        .await
    {
        Ok(_) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[delete("/users/{id}")]
async fn delete_user(data: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();

    // Clone ID for the closure
    let id_clone = id.clone();

    match data
        .db_operation(move |db| db.remove(&id_clone))
        .await
    {
        Ok(Some(_)) => HttpResponse::NoContent().finish(),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let db = sled::open("my_db").expect("Failed to open database");
    let db = Arc::new(Mutex::new(db));

    let workers = num_cpus::get() * 2;
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState { db: Arc::clone(&db) }))
            .service(create_user)
            .service(get_user)
            .service(update_user)
            .service(delete_user)
    })
    .workers(workers)
    .bind(("0.0.0.0", port.parse().unwrap()))?
    .run()
    .await
}