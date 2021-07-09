use crate::RetryAfter;
use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize)]
struct Resp<'a> {
    message: &'a str,
}

#[derive(Serialize)]
struct AuthResp {
    auth: bool,
    ratelimited: bool,
    premium: bool,
    ratelimit: i32,
    left: i32,
    after: u64,
}

#[get("/dbdata")]
async fn get_key(pool: web::Data<sqlx::postgres::PgPool>) -> impl Responder {
    let res: Option<(String,)> = sqlx::query_as(r#"SELECT apikey FROM tokens"#)
        .fetch_optional(&**pool)
        .await
        .unwrap();
    let fres = match res {
        Some(val) => val,
        None => panic!("UWU WHAT NO RESULTS SMH"),
    };
    HttpResponse::Ok().body(fres.0)
}
#[get("/addkey/{token}/{userid}")]
async fn add_key(
    token: web::Path<(String, i64)>,
    pool: web::Data<sqlx::postgres::PgPool>,
) -> impl Responder {
    let deets = token.into_inner();
    let res = sqlx::query(r#"INSERT INTO TOKENS (userid, apikey, uses, totaluses, ratelimit, enhanced) VALUES ($1,$2,1,1,60,'f')"#)
        .bind(deets.1)
        .bind(deets.0)
        .execute(&**pool)
        .await;
    match res {
        Ok(_val) => {
            return HttpResponse::Ok().json(Resp {
                message: "ADDED TOKEN",
            })
        }
        Err(_error) => {
            return HttpResponse::InternalServerError().json(Resp {
                message: "UNABLE TO ADD TOKEN",
            })
        }
    }
}

#[get("/changelimits/{token}/{limit}")]
async fn update_limit(
    token: web::Path<(String, i16)>,
    pool: web::Data<sqlx::postgres::PgPool>,
) -> impl Responder {
    let deets = token.into_inner();
    let res = sqlx::query(
        r#"                                                                       
    UPDATE tokens                                                                 
    SET "ratelimit"=$1                                                              
    WHERE "apikey"=$2;"#,
    )
    .bind(deets.1)
    .bind(deets.0)
    .execute(&**pool)
    .await;
    match res {
        Ok(_val) => {
            return HttpResponse::Ok().json(Resp {
                message: "NEW RATELIMITS SET",
            })
        }
        Err(_error) => {
            return HttpResponse::InternalServerError().json(Resp {
                message: "UNABLE TO UPATE LIMITS",
            })
        }
    }
}

#[get("/resetkey/{token}/{userid}")]
async fn reset_key(
    token: web::Path<(String, i64)>,
    pool: web::Data<sqlx::postgres::PgPool>,
) -> impl Responder {
    let deets = token.into_inner();
    let res = sqlx::query(
        r#"
    UPDATE tokens
    SET "apikey"=$1
    WHERE "userid"=$2;"#,
    )
    .bind(deets.0)
    .bind(deets.1)
    .execute(&**pool)
    .await;
    match res {
        Ok(_val) => {
            return HttpResponse::Ok().json(Resp {
                message: "RESET TOKEN",
            })
        }
        Err(_error) => {
            return HttpResponse::InternalServerError().json(Resp {
                message: "UNABLE TO RESET TOKEN",
            })
        }
    }
}

#[get("/deletekey/{token}")]
async fn delete_key(
    token: web::Path<String>,
    pool: web::Data<sqlx::postgres::PgPool>,
) -> impl Responder {
    let apitoken = token.into_inner();
    let res = sqlx::query(r#"DELETE FROM tokens WHERE "apikey"=$1"#)
        .bind(apitoken)
        .execute(&**pool)
        .await;
    match res {
        Ok(_val) => {
            return HttpResponse::Ok().json(Resp {
                message: "ADDED TOKEN",
            })
        }
        Err(_error) => {
            return HttpResponse::InternalServerError().json(Resp {
                message: "UNABLE TO ADD TOKEN",
            })
        }
    }
}

#[get("/auth/{key}")]
async fn get_data(
    key: web::Path<String>,
    pool: web::Data<sqlx::postgres::PgPool>,
    dat: web::Data<Arc<Mutex<RetryAfter>>>,
) -> impl Responder {
    let apikey = key.into_inner();
    let mres: Option<(i32, i32)> = sqlx::query_as(
        r#"
UPDATE tokens
SET "uses" = "uses" + 1,"totaluses" = "totaluses" + 1
WHERE "apikey"=$1
RETURNING uses, "ratelimit";
    "#,
    )
    .bind(&apikey)
    .fetch_optional(&**pool)
    .await
    .unwrap();
    let after_val_r = dat.lock().await;
    let after_val = &after_val_r.timestamp;
    // let after_val = rx.lo.await.unwrap_or_else(|| "None".to_string());
    println!("After: {}", after_val);
    match mres {
        Some(val) => {
            return HttpResponse::Ok().json(AuthResp {
                auth: true,
                ratelimited: val.0 >= val.1,
                premium: val.1 > 60 as i32,
                ratelimit: val.1,
                left: val.1 - val.0,
                after: *after_val,
            })
        }
        None => {
            return HttpResponse::Ok().json(AuthResp {
                auth: false,
                ratelimited: false,
                premium: false,
                ratelimit: 0,
                left: 0,
                after: *after_val,
            })
        }
    };
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(get_key);
    config.service(get_data);
    config.service(add_key);
    config.service(delete_key);
    config.service(reset_key);
    config.service(update_limit);
}
