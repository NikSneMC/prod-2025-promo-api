use actix_web::{
    middleware::from_fn,
    web::{scope, ServiceConfig},
};

use crate::{auth::auth_middleware_usr, util::cors::default_cors};

mod activate;
mod comments;
mod get_promo;
mod like;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("{promo_id}")
            .wrap(default_cors())
            .wrap(from_fn(auth_middleware_usr))
            .service(get_promo::get_handler)
            .service(like::post_handler)
            .service(like::delete_handler)
            .configure(comments::config)
            .service(activate::post_handler),
    );
}
