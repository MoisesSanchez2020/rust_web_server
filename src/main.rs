use actix_web::{web, App, HttpResponse, Responder, http::header::HeaderMap, http::header::HeaderValue};
use actix_web::middleware::Logger;
use actix_web::web::Data;
use env_logger::Env;
use reqwest::Client;
use serde_json::json;
use std::env;
use log::error; // Add this line

#[derive(Debug)]
struct CustomError {
    status: u16,
    message: String,
}

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "CustomError(status: {}, message: {})", self.status, self.message)
    }
}

// Implement ResponseError trait for CustomError
impl actix_web::ResponseError for CustomError {}

// Define yelp function to make a request to Yelp API
async fn yelp() -> impl Responder {
    let client = Client::new();
    let api_key = env::var("YELP_API_KEY").unwrap_or_else(|_| "Ek8zOvr4GbmeYX5mIVVuxEIcd9Xphhcm_KujQv_yiZsvB9ZcjSJ-XRlKqw0v0YHXuaUFZN25M7mvuSt6NHS644Af9cBrhSiAF5BtncZPZTvOvoXQCuogcMmU2DFEZXYx".to_string());
    let response = client.get("https://api.yelp.com/v3/businesses/search")
     .header("Authorization", format!("Bearer {}", api_key))
     .query(&[("location", "San Mateo"), ("term", "restaurants")])
     .send()
     .await;

    match response {
        Ok(response) => {
            match response.text().await {
                Ok(text) => {
                    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
                    HttpResponse::Ok()
                        .content_type("text/html")
                        .body(format!(r#"<!DOCTYPE html>
<html>
<head>
    <title>Yelp API Response</title>
</head>
<body>
    <pre>{}</pre>
</body>
</html>"#, json))
                },
                Err(e) => {
                    error!("Error reading response text: {}", e);
                    HttpResponse::InternalServerError().into()
                },
            }
        }
        Err(e) => {
            error!("Error making request: {}", e);
            HttpResponse::InternalServerError().into()
        },
    }
}

// Define main function to start the server
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let server = actix_web::HttpServer::new(move || {
        App::new()
          .wrap(Logger::default())
          .data(Client::new())
          .route("/", web::get().to(yelp))
    })
  .bind("127.0.0.1:8080")?
  .run()
  .await;

    server
}