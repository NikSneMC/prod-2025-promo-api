use actix_files::NamedFile;
use actix_web::{get, Result};

#[get("ping")]
pub async fn get_handler() -> Result<NamedFile> {
    Ok(NamedFile::open("./assets/ping.html")?)
}
