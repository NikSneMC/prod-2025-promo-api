use actix_web::{
    middleware::from_fn,
    web::{scope, ServiceConfig},
};

use crate::{auth::auth_middleware_usr, util::cors::default_cors};

mod by_id;
mod history;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("promo")
            .wrap(default_cors())
            .wrap(from_fn(auth_middleware_usr))
            .service(history::get_handler)
            .configure(by_id::config),
    );
}
