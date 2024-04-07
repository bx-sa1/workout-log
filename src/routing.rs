use crate::db;
use std::{boxed::Box, collections::HashMap, convert::Infallible, error::{self, Error}, fmt, future::Future, pin::Pin, result};

use http_body_util::BodyExt;
use hyper::{
    body::{Buf, Incoming},
    service::Service,
    Method, Request, Response, StatusCode,
};

type Result<T> = result::Result<T, RouterError<Box<dyn error::Error>>>;

#[derive(Debug)]
pub struct RouterError<E>(Option<E>, StatusCode, &'static str);

impl fmt::Display for RouterError<Box<dyn error::Error>> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let e = format!("{:?}", self.0);

        let ne = e.as_str().replace("\"", "\\\"");


        write!(f, r#"
        {{ 
            "error": "{}",
            "status_code": "{}",
            "message": "{}"
        }}"#, ne, self.1, self.2)

    }
}

#[derive(Clone)]
pub struct Router {
    db: db::AsyncDB,
}

impl Router {
    pub fn new(db: db::AsyncDB) -> Router {
        Self { db }
    }
}

impl Service<Request<Incoming>> for Router {
    type Response = Response<String>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = result::Result<Self::Response, Self::Error>> + Send + Sync>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        fn create_response_json_string(ok: bool, body: String) -> String {
            format!(r#"
                        {{
                            "status": "{}",
                            "result": {}
                        }}"#, if ok { "ok" } else { "err" }, body
                    )
        }

        let s = self.clone();
        Box::pin(async move {
            let res = match (req.method(), req.uri().path()) {
                (&Method::GET, "/workout") => get_workout(req, s.db).await,
                (&Method::POST, "/workout") => add_workout(req, s.db).await,
                (&Method::PUT, "/workout") => update_workout(req, s.db).await,
                (&Method::DELETE, "/workout") => delete_workout(req, s.db).await,

                (&Method::GET, "/workouts") => get_workouts(req, s.db).await,

                _ => Err(RouterError(None, StatusCode::NOT_FOUND, "Not a valid endpoint")),
            };

            match res {
                Ok(o) => Ok(Response::new(create_response_json_string(true, o))),
                Err(e) => Ok(Response::builder()
                    .status(e.1)
                    .body(create_response_json_string(false, format!("{}", e)))
                    .unwrap())
            }
        })
    }
}

fn get_uri_param(req: &Request<Incoming>, key: &str) -> Option<(String, String)> {
    req.uri()
    .query()
    .map(|v| {
        url::form_urlencoded::parse(v.as_bytes())
            .into_owned()
            .collect()
    })
    .unwrap_or_else(HashMap::new)
    .into_iter()
    .find(|(k, _)| *k == key)
}

async fn collect_req_body(req: Request<Incoming>) -> Option<impl Buf> {
    Some(match req.collect().await {
       Ok(o) => o,
       Err(_) => return None
    }.aggregate())
}

async fn get_workout(req: Request<Incoming>, db: db::AsyncDB) -> Result<String> {
    let date = match get_uri_param(&req, "date") {
        Some((_, date)) => date,
        None => {
            return Err(RouterError(None, StatusCode::BAD_REQUEST, "Bad date request"))
        }
    };

    let workout = match db.lock().unwrap().get_workout(date) {
        Ok(o) => o,
        Err(e) => {
            return Err(RouterError(Some(e.into()), StatusCode::INTERNAL_SERVER_ERROR, "Can't find date in DB"))
        }
    };

    let json = match serde_json::to_string_pretty(&workout) {
        Ok(o) => o,
        Err(e) => {
            return Err(RouterError(Some(e.into()), StatusCode::BAD_REQUEST, "Can't serialize workout to json"))
        }
    };

    Ok(json)
}

async fn add_workout(req: Request<Incoming>, db: db::AsyncDB) -> Result<String> {
    let whole_body = match collect_req_body(req).await {
        Some(b) => b,
        None => {
            return Err(RouterError(None, StatusCode::INTERNAL_SERVER_ERROR, "Failed to collect request body"))
        }
    };

    let workout: db::Workout = match serde_json::from_reader(&mut whole_body.reader()) {
        Ok(o) => o,
        Err(e) => {
            return Err(RouterError(Some(e.into()), StatusCode::UNSUPPORTED_MEDIA_TYPE, "Failed to parse json request"))
        }
    };
    
    match db.lock().unwrap().add_workout(workout) {
        Ok(_) => {},
        Err(e) => return Err(RouterError(Some(e.into()), StatusCode::INTERNAL_SERVER_ERROR, "Failed to add workout to db"))
    };

    Ok("\"success\"".to_string())
}

async fn update_workout(req: Request<Incoming>, db: db::AsyncDB) -> Result<String> {
    let date = match get_uri_param(&req, "date") {
        Some((_, date)) => date,
        None => {
            return Err(RouterError(None, StatusCode::BAD_REQUEST, "Bad date request"))
        }
    };

    let whole_body = match collect_req_body(req).await {
        Some(b) => b,
        None => {
            return Err(RouterError(None, StatusCode::INTERNAL_SERVER_ERROR, "Failed to collect request body"))
        }
    };

    let new_workout: db::Workout = match serde_json::from_reader(&mut whole_body.reader()) {
        Ok(o) => o,
        Err(e) => {
            return Err(RouterError(Some(e.into()), StatusCode::UNSUPPORTED_MEDIA_TYPE, "Failed to parse json request"))
        }
    };

    match db.lock().unwrap().update_workout(date, new_workout) {
        Ok(_) => {},
        Err(e) => {
            return Err(RouterError(Some(e.into()), StatusCode::INTERNAL_SERVER_ERROR, "Failed to update db"))
        }
    }

    Ok("\"success\'".to_string())
}

async fn delete_workout(req: Request<Incoming>, db: db::AsyncDB) -> Result<String> {
    let date = match get_uri_param(&req, "date") {
        Some((_, date)) => date,
        None => {
            return Err(RouterError(None, StatusCode::BAD_REQUEST, "Bad date request"))
        }
    };

    match db.lock().unwrap().delete_workout(date) {
        Ok(_) => {},
        Err(e) => {
            return Err(RouterError(Some(e.into()), StatusCode::INTERNAL_SERVER_ERROR, "Failed to update db"))
        }
    }

    Ok("\"success\"".to_string())
}

async fn get_workouts(req: Request<Incoming>, db: db::AsyncDB) -> Result<String> {
    let limit = match get_uri_param(&req, "limit") {
        Some((_, limit)) => limit,
        None => "300".to_string()
    };

    let limit = match limit.parse::<i64>() {
        Ok(o) => o,
        Err(e) => return Err(RouterError(Some(e.into()), StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse limit param; not an integer"))
    };

    let workout_list = match db.lock().unwrap().get_workouts(limit) {
        Ok(o) => o,
        Err(e) => {
            return Err(RouterError(Some(e.into()), StatusCode::INTERNAL_SERVER_ERROR, "Can't get workouts fro, DB"))
        }
    };

    let json = match serde_json::to_string_pretty(&workout_list) {
        Ok(o) => o,
        Err(e) => {
            return Err(RouterError(Some(e.into()), StatusCode::BAD_REQUEST, "Can't serialize workouts to json"))
        }
    };

    Ok(json)
}
