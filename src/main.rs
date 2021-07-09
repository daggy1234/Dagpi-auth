use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer, Responder};

use env_logger;
use std::convert::TryInto;
use std::sync::{atomic, Arc};
use std::time::Duration;
mod routing;
use dotenv;
use tokio::time;
mod stats;
use chrono::offset::Utc;
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use std::time::{SystemTime, UNIX_EPOCH};
mod auth;

use stats::StatPool;

#[derive(Serialize)]
struct ErrorResp<'a> {
    message: &'a str,
}

pub async fn resp_not_found() -> HttpResponse {
    HttpResponse::NotFound().json(ErrorResp {
        message: "Page not found",
    })
}

async fn greet(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().json(ErrorResp {
        message: "HELLO WORLD",
    })
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    std::env::set_var("RUST_BACKTRACE", "1");
    dotenv::dotenv().ok();

    let tstr = Arc::new(atomic::AtomicU64::new(0_u64));

    let pool_tstr = Arc::clone(&tstr);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&std::env::var("DATABASE_URL_MAIN").expect("DATABASE_URL_MAIN not set"))
        .await
        .unwrap();
    let sp = PgPoolOptions::new()
        .max_connections(10)
        .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL not set"))
        .await
        .unwrap();
    let st = StatPool { pool: sp };
    let new_st = pool.clone();
    sqlx::query("UPDATE TOKENS SET uses = 0;")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("SELECT * FROM stats;")
        .execute(&st.pool)
        .await
        .unwrap();
    env_logger::init();
    let server = HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(st.clone())
            .data(tstr.clone())
            .route("/", web::get().to(greet))
            .wrap(auth::RequiresAuth)
            .configure(routing::init_routes)
            .configure(stats::init_routes)
            .default_service(web::route().to(resp_not_found))
            .wrap(middleware::Logger::default())
    })
    .bind("0.0.0.0:8000")?
    .run();

    actix_web::rt::spawn(async move {
        let mut interval = time::interval(Duration::from_millis(60000));
        loop {
            interval.tick().await;
            sqlx::query("UPDATE TOKENS SET uses = 0;")
                .execute(&new_st)
                .await
                .unwrap();
            let start_utc = Utc::now().timestamp();
            let start: u64 = (start_utc + 60).try_into().unwrap();
            pool_tstr.store(start, atomic::Ordering::Relaxed);
        }
    });

    server.await
}
