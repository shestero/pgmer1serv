use actix_web::*;
use nng::{Message, Protocol, Socket};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Debug;
//use actix_web::error::ErrorInternalServerError;

const SERVICE_HOST: &str = "127.0.0.1";
const SERVICE_PORT: u16 = 8000;
const SERVICE_URL: &str = "tcp://127.0.0.1:10234"; // uplink

#[derive(Debug, Serialize, Deserialize)]
struct Edge {
    src: String,
    dest: String,
    weight: f64,
}
#[derive(Clone, Serialize, Deserialize)]
struct NodeScore {
    node: String,
    ego: String,
    score: f64,
}

fn request<T: for<'a> Deserialize<'a>>(
    req: &Vec<u8>,
) -> core::result::Result<T, Box<dyn std::error::Error + 'static>> {
    let client = Socket::new(Protocol::Req0)?;
    client.dial(SERVICE_URL)?;
    client
        .send(Message::from(req.as_slice()))
        .map_err(|(_, err)| err)?;
    let msg: Message = client.recv()?;
    let slice: &[u8] = msg.as_slice();

    rmp_serde::from_slice(slice).or_else(|_| {
        let err: String = rmp_serde::from_slice(slice)?;
        Err(Box::from(format!("Server error: {}", err)))
    })
}

#[get("/")]
async fn service_url() -> impl Responder {
    let body = format!(
        "<html>\n\
        <head>\
        <meta charset=\"UTF-8\" />\
        </head>\n\
        <body>\n\
        <pre>SERVICE_URL = {}</pre>\n<hr>\n\
        <h3>API examples</h3>\n\
        <ul>\n\
            <li><a href=\"/edge/a1/a2/3\" target=\"_new\">/edge/a1/a2/3</a></li>\n\
            <li><a href=\"/node_score/a1/a2\" target=\"_new\">/node_score/a1/a2</a></li>\n\
            <li><a href=\"/scores/a1\" target=\"_new\">/scores/a1</a></li>\n\
        </ul>\n\
        </body>\n\
        </html>\n",
        SERVICE_URL
    );
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body)
}

fn edge(e: &Edge) -> Result<impl Responder> {
    let rq = ((e,), ());
    let req = rmp_serde::to_vec(&rq).expect("cannot encode");
    let res: Vec<NodeScore> = request(&req)?;
    Ok(web::Json(res))
}

#[get("/edge/{src}/{dest}/{weight}")]
async fn get_edge(path: web::Path<Edge>) -> Result<impl Responder> {
    let e: Edge = path.into_inner();
    println!("[GET] edge: {:?}", e);
    edge(&e)
}

#[put("/edge")]
async fn put_edge(body: web::Json<Edge>) -> Result<impl Responder> {
    let e: Edge = body.into_inner();
    println!("[PUT] edge: {:?}", e);
    let rq = ((e,), ());
    let req = rmp_serde::to_vec(&rq).expect("cannot encode");
    let res: Vec<NodeScore> = request(&req)?;
    let json = json!({
    "message": res,
    });
    Ok(web::Json(json))
}

#[get("/node_score/{ego}/{target}")]
async fn node_score(path: web::Path<(String, String)>) -> Result<impl Responder> {
    let (ego, target) = path.into_inner();
    println!("node score? {} --> {}", ego, target);
    let rq = ((("src", "=", ego), ("dest", "=", target)), ());
    let req = rmp_serde::to_vec(&rq).expect("cannot encode");
    let res: Vec<NodeScore> = request(&req)?;
    let first: NodeScore = res.first().unwrap().clone(); // ?
    Ok(web::Json(first))
}

#[get("/scores/{ego}")]
async fn scores(path: web::Path<String>) -> Result<impl Responder> {
    let ego: String = path.into_inner();
    println!("scores? {}", ego);
    let rq = ((("src", "=", ego),), ());
    let req = rmp_serde::to_vec(&rq).expect("cannot encode");
    let res: Vec<NodeScore> = request(&req)?;
    Ok(web::Json(res))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("pgmer1serv binds {SERVICE_HOST}:{SERVICE_PORT}, listen {SERVICE_URL}");
    HttpServer::new(|| {
        App::new()
            .service(service_url)
            .service(get_edge)
            .service(put_edge)
            .service(node_score)
            .service(scores)
    })
    .bind((SERVICE_HOST, SERVICE_PORT))?
    .run()
    .await
}
