use actix_web::web::{scope, ServiceConfig};

use crate::util::cors::default_cors;

mod sign_in;
mod sign_up;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("auth")
            .wrap(default_cors())
            .service(sign_up::post_handler)
            .service(sign_in::post_handler),
    );
}
