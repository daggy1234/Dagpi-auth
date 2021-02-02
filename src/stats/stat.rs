use crate::stats::model::*;
use actix_web::{get, post, web, HttpResponse, Responder};
use sqlx;

#[get("/stats/{apikey}")]
async fn get_stat(key: web::Path<String>, pool: web::Data<StatPool>) -> impl Responder {
    let k = key.into_inner();
    let p = &pool.into_inner().pool;
    let qr = sqlx::query!(
        "
        SELECT route, agent, api, time FROM stats
        WHERE apikey = $1 AND time > NOW() - INTERVAL '24 hours';
    ",
        k
    )
    .fetch_all(p)
    .await;
    let q = match qr {
        Ok(v) => v,
        Err(_r) => return HttpResponse::InternalServerError().json(Resp { message: "error" }),
    };

    let out: Vec<Stat> = q
        .into_iter()
        .map(|i| -> Stat {
            Stat {
                route: String::from(i.route),
                user_agent: String::from(i.agent),
                api: String::from(i.api),
                timestamp: i.time.unix_timestamp(),
            }
        })
        .collect();
    let c = StatResp {
        total: out.len(),
        data: out,
    };
    return HttpResponse::Ok().json(c);
}

#[post("/statpost")]
async fn post_stat(form: web::Json<StatForm>, pool: web::Data<StatPool>) -> impl Responder {
    let p = &pool.into_inner().pool;
    let f = form.into_inner();
    let qr = sqlx::query!(
        "
        INSERT INTO stats(time, apikey, route, agent, api)
        VALUES (NOW(),$1,$2,$3,$4);
    ",
        f.token,
        f.route,
        f.user_agent,
        f.api
    )
    .execute(p)
    .await;
    match qr {
        Ok(_r) => HttpResponse::Ok().json(Resp {
            message: "Added Stats",
        }),
        Err(_r) => HttpResponse::InternalServerError().json(Resp {
            message: "Unable to add Stats",
        }),
    }
}
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(get_stat);
    config.service(post_stat);
}
