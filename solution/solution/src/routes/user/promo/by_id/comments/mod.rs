use actix_web::{
    middleware::from_fn,
    web::{scope, ServiceConfig},
};

use crate::{auth::auth_middleware_usr, util::cors::default_cors};

mod by_id;
mod get_comments;
mod post_comment;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("comments")
            .wrap(default_cors())
            .wrap(from_fn(auth_middleware_usr))
            .service(post_comment::post_handler)
            .service(get_comments::get_handler)
            .service(by_id::get_handler)
            .service(by_id::put_handler)
            .service(by_id::delete_handler),
    );
}
