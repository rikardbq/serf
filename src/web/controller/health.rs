use actix_web::{get, web, HttpResponse, Responder};

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(handle_health_get);
}

#[get("/health")]
async fn handle_health_get() -> impl Responder {
    HttpResponse::Ok().body("OK")
}
