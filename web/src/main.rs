use actix_files as fs;
use actix_web::{web, App, HttpRequest, HttpServer, Responder};

// async fn greet(req: HttpRequest) -> impl Responder {
//     let name = req.match_info().get("name").unwrap_or("World");
//     format!("Hello {}!", &name)
// }

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            // List all plots at `/plots`
            .service(fs::Files::new("/plots", "../plotter/data/plots").show_files_listing())
            // List all points at `/points`
            .service(fs::Files::new("/points", "../plotter/data/points").show_files_listing())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
