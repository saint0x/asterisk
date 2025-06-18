use actix_web::{get, web, App, HttpServer, Responder, HttpResponse, HttpRequest};


async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Welcome to the Asterisk server!")
}

#[get("/hello/{name}")]
async fn hello(name: web::Path<String>) -> impl Responder {
    HttpResponse::Ok().body(format!("Hello, {}!", name))
}

#[get("/json")]
async fn json_data() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "message": "This is a JSON response",
        "data": [1, 2, 3, 4, 5]
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server_address = "127.0.0.1:8080";
    println!("Starting server at http://{}", server_address);

    HttpServer::new(move || {
        App::new()
            .service(index)
            .service(hello)
            .service(json_data)
            .route("/health", web::get().to(health_check))
            .route("/whoami", web::get().to(whoami_with_ip_extraction))
    })
    .bind(server_address)?
    .run()
    .await
}

// New handler to extract client IP
async fn whoami_with_ip_extraction(req: HttpRequest) -> impl Responder {
    if let Some(client_addr) = req.peer_addr() {
        HttpResponse::Ok().body(format!("Client IP: {}", client_addr.ip()))
    } else {
        HttpResponse::InternalServerError().body("Could not determine client IP")
    }
} 