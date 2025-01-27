use actix_web::web::{scope, ServiceConfig};

use crate::util::cors::default_cors;

mod auth;
mod feed;
mod profile;
mod promo;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("user")
            .wrap(default_cors())
            .configure(auth::config)
            .configure(profile::config)
            .configure(feed::config)
            .configure(promo::config),
    );
}
