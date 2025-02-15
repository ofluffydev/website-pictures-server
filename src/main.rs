#![windows_subsystem = "windows"]

use actix_web::{get, web, HttpResponse, Responder, http::header};
use actix_cors::Cors;
use log::{error, info};
use serde_json::json;
use std::fs;
use std::path::Path;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC, percent_decode_str};

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
                        let encoded_name = utf8_percent_encode(name, NON_ALPHANUMERIC).to_string();
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
                        galleries.push(json!({ "name": encoded_name, "images": images }));
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
    let (encoded_gallery_name, image_name) = path.into_inner();
    let gallery_name = percent_decode_str(&encoded_gallery_name).decode_utf8_lossy().to_string();
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
    let encoded_gallery_name = path.into_inner();
    let gallery_name = percent_decode_str(&encoded_gallery_name).decode_utf8_lossy().to_string();
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_default_env()
        .filter(None, log::LevelFilter::Debug)
        .init();

    info!("Starting server..");

    let photos_path = Path::new(PHOTOS_PATH);
    if !photos_path.exists() {
        panic!("Photos path does not exist: {}", PHOTOS_PATH);
    }

    actix_web::HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
            .allowed_header(header::CONTENT_TYPE)
            .max_age(3600);

        actix_web::App::new()
            .wrap(cors)
            .service(index)
            .service(get_galleries)
            .service(get_highlight)
            .service(get_image)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
