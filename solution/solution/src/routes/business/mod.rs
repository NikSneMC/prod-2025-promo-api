use actix_web::web::{scope, ServiceConfig};

use crate::util::cors::default_cors;

mod auth;
mod promo;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("business")
            .wrap(default_cors())
            .configure(auth::config)
            .configure(promo::config),
    );
}
