use crate::lattice::affine;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct BuildRequest {
    pub repo: String,
    pub db: String,
}

#[derive(Deserialize)]
pub struct QueryRequest {
    pub zoom: String,
    pub db: String,
}

#[derive(Serialize)]
pub struct ApiResponse {
    pub status: String,
    pub message: String,
}

#[post("/build")]
async fn build_lattice(req: web::Json<BuildRequest>) -> impl Responder {
    println!("🚀 API: Reticulating repo: {}", req.repo);
    affine::build_lattice(&req.repo, &req.db);
    HttpResponse::Ok().json(ApiResponse {
        status: "success".to_string(),
        message: format!("Lattice built for {}", req.repo),
    })
}

#[get("/zoom/{node_id}")]
async fn zoom_node(path: web::Path<String>, query: web::Query<QueryRequest>) -> impl Responder {
    let node_id = path.into_inner();
    println!("🔍 API: Zooming into node: {}", node_id);
    affine::query_lattice(&node_id, &query.db);
    HttpResponse::Ok().json(ApiResponse {
        status: "success".to_string(),
        message: format!("Zoomed into node {}", node_id),
    })
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("Git-Reticulator API is healthy.")
}

pub async fn start_server(db_uri: String) -> std::io::Result<()> {
    println!("🌐 Git-Reticulator API starting on http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_uri.clone()))
            .service(build_lattice)
            .service(zoom_node)
            .service(health)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
