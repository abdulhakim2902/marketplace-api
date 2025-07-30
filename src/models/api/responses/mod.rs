pub mod attribute;
pub mod collection_offer;

use serde::Serialize;

#[derive(Serialize)]
pub struct HttpResponse<T> {
    pub data: T,
}

#[derive(Serialize)]
pub struct HttpResponsePaging<T> {
    pub data: T,
    pub total: i64,
}

#[derive(Serialize)]
pub struct HttpResponseErr {
    pub code: String,
    pub msg: String,
}

#[derive(Serialize)]
pub struct HttpResponseConstErr {
    pub code: &'static str,
    pub msg: &'static str,
}

pub const INTERNAL_SERVER_ERR: HttpResponseConstErr = HttpResponseConstErr {
    code: "ERR_000",
    msg: "Internal Server Error",
};

pub const NOT_FOUND_ERR: HttpResponseConstErr = HttpResponseConstErr {
    code: "ERR_404",
    msg: "Not Found",
};

#[derive(Serialize)]
pub struct ResponsePaging<T> {
    pub data: T,
    pub total: i64,
}

#[derive(Serialize)]
pub struct MessageResponse<T> {
    pub id: String,
    pub data: T,
    pub ttl: Option<i64>,
}

#[derive(Serialize)]
pub struct ResponseErr {
    pub code: i64,
    pub msg: String,
}
