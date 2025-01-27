use actix_web::{
    middleware::from_fn,
    web::{scope, ServiceConfig},
};

use crate::{auth::auth_middleware_usr, util::cors::default_cors};

mod edit_profile;
mod get_profile;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("profile")
            .wrap(default_cors())
            .wrap(from_fn(auth_middleware_usr))
            .service(get_profile::get_handler)
            .service(edit_profile::patch_handler),
    );
}
