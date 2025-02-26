#![windows_subsystem = "windows"]

use actix_cors::Cors;
use actix_web::{get, http::header, web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use dotenv::dotenv;
use log::{error, info};
use serde_json::json;
use std::fs;
use std::path::Path;
use website_pictures_server::db::{get_database, get_services};

const PHOTOS_PATH: &str = "Photos";

/// Simple ping at index
#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Pong")
}

/// Returns a JSON list of all galleries.
#[get("/galleries")]
async fn get_galleries() -> impl Responder {
    let photos_path = Path::new(PHOTOS_PATH);
    let mut galleries = Vec::new();

    if let Ok(entries) = fs::read_dir(photos_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                if entry.path().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        let mut images = Vec::new();
                        if let Ok(image_entries) = fs::read_dir(entry.path()) {
                            for image_entry in image_entries {
                                if let Ok(image_entry) = image_entry {
                                    if image_entry.path().is_file() {
                                        if let Some(image_name) = image_entry.file_name().to_str() {
                                            if !image_name.ends_with(".db") {
                                                images.push(image_name.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        galleries.push(json!({ "name": name, "images": images }));
                    }
                }
            }
        }
    } else {
        error!("Failed to read directory: {}", PHOTOS_PATH);
    }

    info!("Galleries found: {:?}", galleries);
    HttpResponse::Ok().json(galleries)
}

/// Returns the actual image file.
#[get("/galleries/{name}/{image}")]
async fn get_image(path: web::Path<(String, String)>) -> impl Responder {
    let (gallery_name, image_name) = path.into_inner();
    let image_path = Path::new(PHOTOS_PATH).join(gallery_name).join(image_name);

    if image_path.exists() && image_path.is_file() {
        match fs::read(image_path) {
            Ok(image_data) => HttpResponse::Ok()
                .content_type("image/jpeg")
                .body(image_data),
            Err(_) => HttpResponse::InternalServerError().body("Error reading image file"),
        }
    } else {
        HttpResponse::NotFound().body("Image not found")
    }
}

/// Returns the highlight (first image in folder) of the gallery.
#[get("/galleries/{name}/highlight")]
async fn get_highlight(path: web::Path<String>) -> impl Responder {
    let gallery_name = path.into_inner();
    let gallery_path = Path::new(PHOTOS_PATH).join(gallery_name);

    if gallery_path.exists() && gallery_path.is_dir() {
        let mut images = Vec::new();
        if let Ok(image_entries) = fs::read_dir(gallery_path) {
            for image_entry in image_entries {
                if let Ok(image_entry) = image_entry {
                    if image_entry.path().is_file() {
                        if let Some(_image_name) = image_entry.file_name().to_str() {
                            images.push(image_entry.path());
                        }
                    }
                }
            }
        }
        if images.is_empty() {
            HttpResponse::NotFound().body("No images found in gallery")
        } else {
            let highlight_path = images[0].clone();
            match fs::read(highlight_path) {
                Ok(image_data) => HttpResponse::Ok()
                    .content_type("image/jpeg")
                    .body(image_data),
                Err(_) => HttpResponse::InternalServerError().body("Error reading highlight image"),
            }
        }
    } else {
        HttpResponse::NotFound().body("Gallery not found")
    }
}

/// Returns a JSON list of all services.
#[get("/services")]
async fn get_services_endpoint(pool: web::Data<Pool>) -> impl Responder {
    match get_services(&pool).await {
        Ok(services) => HttpResponse::Ok().json(services),
        Err(e) => {
            error!("Failed to get services: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to get services")
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server..");
    dotenv().ok();

    ::std::env::set_var("RUST_LOG", "debug");
    ::std::env::set_var("actix_web", "debug");
    env_logger::Builder::from_default_env()
        .filter(None, log::LevelFilter::Debug)
        .init();

    let photos_path = Path::new(PHOTOS_PATH);
    if !photos_path.exists() {
        panic!("Photos path does not exist: {}", PHOTOS_PATH);
    }

    let pool = match get_database().await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database connection pool: {:?}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to get database connection pool",
            ));
        }
    };

    actix_web::HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
            .allowed_header(header::CONTENT_TYPE)
            .max_age(3600);

        actix_web::App::new()
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .service(index)
            .service(get_galleries)
            .service(get_highlight)
            .service(get_image)
            .service(get_services_endpoint)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
