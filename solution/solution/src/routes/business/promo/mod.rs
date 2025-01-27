use actix_web::{
    middleware::from_fn,
    web::{scope, ServiceConfig},
};

use crate::{auth::auth_middleware_cmp, util::cors::default_cors};

mod by_id;
mod create_promo;
mod list_promos;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("promo")
            .wrap(default_cors())
            .wrap(from_fn(auth_middleware_cmp))
            .service(create_promo::post_handler)
            .service(list_promos::get_handler)
            .configure(by_id::config),
    );
}
