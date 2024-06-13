use actix_web::error::InternalError;
use actix_web::http::StatusCode;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use sailfish::TemplateSimple;

#[derive(TemplateSimple)]
#[template(path = "actix.stpl")]
struct Greet<'a> {
    name: &'a str,
}

async fn greet(req: HttpRequest) -> actix_web::Result<HttpResponse> {
    let name = req.match_info().get("name").unwrap_or("World");
    let body = Greet { name }
        .render_once()
        .map_err(|e| InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(greet))
            .route("/{name}", web::get().to(greet))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
