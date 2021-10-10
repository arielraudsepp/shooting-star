//! main.rs

use shooting_star::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    run().await
}
