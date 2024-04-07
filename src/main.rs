use actix_web::{web, App, HttpResponse, Responder, http::header::HeaderMap, http::header::HeaderValue, HttpRequest}; // Import necessary modules from Actix web framework
use actix_web::middleware::Logger; // Import Logger middleware for logging
use actix_web::web::Data; // Import Data struct for sharing data between handlers
use actix_files as fs; // Import actix_files for serving static files
use askama::Template; // Import Template trait from Askama for template rendering
use env_logger::Env; // Import Env struct from env_logger crate for environment logging
use reqwest::Client; // Import Client struct from reqwest for making HTTP requests
use serde_json::json; // Import json macro from serde_json for creating JSON objects
use std::env; // Import env module from standard library for environment variables
use log::error; // Import error macro from log crate for logging errors

// Define a struct to hold the data to be passed to the template
#[derive(Template)]
#[template(path = "index.html")] // Specify the path to the template file
struct IndexTemplate {
    message: String,
    is_featured: bool,
    items: Vec<String>,
    json_data: String,
}

// Define yelp function to make a request to Yelp API
async fn yelp() -> impl Responder {
    let client = Client::new(); // Create a new reqwest client
    let api_key = env::var("YELP_API_KEY").unwrap_or_else(|_| "Your_API_Key_Here".to_string()); // Get Yelp API key from environment variables
    let response = client.get("https://api.yelp.com/v3/businesses/search") // Send GET request to Yelp API
        .header("Authorization", format!("Bearer {}", api_key)) // Add Authorization header with API key
        .query(&[("location", "San Mateo"), ("term", "restaurants")]) // Add query parameters
        .send() // Send the request
        .await; // Await for the response

    match response {
        Ok(response) => { // If the response is successful
            match response.text().await { // Read the response body as text
                Ok(text) => {
                    let json_data = text.clone(); // Clone the JSON data
                    let template = IndexTemplate { // Create an instance of IndexTemplate struct
                        message: "Welcome to my Rust web server!".to_string(), // Set message field
                        is_featured: true, // Set is_featured field
                        items: vec!["Item 1".to_string(), "Item 2".to_string(), "Item 3".to_string()], // Set items field
                        json_data: json_data, // Pass the JSON data to the template
                    };
                    HttpResponse::Ok() // Return OK response
                        .content_type("text/html") // Set content type as HTML
                        .body(template.render().unwrap_or_else(|e| format!("Template rendering error: {}", e))) // Render the template
                },
                Err(e) => {
                    error!("Error reading response text: {}", e); // Log error if reading response text fails
                    HttpResponse::InternalServerError().finish() // Return internal server error response
                },
            }
        }
        Err(e) => {
            error!("Error making request: {}", e); // Log error if making request fails
            HttpResponse::InternalServerError().finish() // Return internal server error response
        },
    }
}

// Define main function to start the server
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok(); // Load environment variables from .env file
    env_logger::init_from_env(Env::default().default_filter_or("info")); // Initialize logger from environment

    let server = actix_web::HttpServer::new(move || { // Create Actix web server instance
        App::new()
          .wrap(Logger::default()) // Add default logger middleware
          .service(web::resource("/").route(web::get().to(yelp))) // Map the root URL to the yelp function
          .service(fs::Files::new("/", "./static").index_file("index.html")) // Serve static files
    })
    .bind("127.0.0.1:8080")? // Bind the server to localhost on port 8080
    .run() // Run the server
    .await; // Await for the server to run

    server // Return the server instance
}
