use actix_web::{body::EitherBody, web::Json, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

mod antifraud;
mod comment;
mod company;
mod promo;
mod stats;
mod token;
mod user;

pub use antifraud::{AntiFraudRequest, AntiFraudResponse};
pub use comment::{Comment, CommentPath};
pub use company::Company;
pub use promo::{Promo, PromoPath, PromoTarget, SortPromosBy, UserPromo};
pub use stats::{PromoStats, PromoStatsCountry};
pub use token::{Token, TokenType};
pub use user::{User, UserTargetSettings};

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiError<'a> {
    pub error: &'a str,
    pub description: String,
}

#[derive(Serialize, Debug)]
pub struct EmptyResponse {
    status: String,
}

impl Default for EmptyResponse {
    fn default() -> Self {
        Self {
            status: "ok".to_string(),
        }
    }
}

impl Responder for EmptyResponse {
    type Body = EitherBody<String>;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        Json(self).respond_to(req)
    }
}
