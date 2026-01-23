use actix_web::{App, HttpServer, web};
use sqlx::postgres::PgPoolOptions;

mod context;

pub use context::Context;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://admin:admin@localhost:5432/main".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    sqlx::migrate!("../merc-storage/migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let ctx = Context::new(pool);

    println!("Starting server at http://127.0.0.1:3000");

    HttpServer::new(move || App::new().app_data(web::Data::new(ctx.clone())))
        .bind(("127.0.0.1", 3000))?
        .run()
        .await
}
