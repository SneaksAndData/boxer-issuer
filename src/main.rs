mod filters;
mod services;

use actix_web::{get, web, App, HttpServer};
use filters::oauth_filter::ExternalTokenMiddlewareFactory;
use std::io::Result;

#[get("/token/{identity_provider}")]
async fn token(identity_provider: web::Path<String>) -> String {
    format!(
        "successfully logged in with {0}",
        identity_provider.as_str()
    )
}
#[actix_web::main]
async fn main() -> Result<()> {
    let addr = ("127.0.0.1", 8080);
    println!("listening on {}:{}", &addr.0, &addr.1);
    HttpServer::new(move || {
        App::new()
            .wrap(ExternalTokenMiddlewareFactory::new())
            .service(token)
    })
    .bind(addr)?
    .run()
    .await
}
