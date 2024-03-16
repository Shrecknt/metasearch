pub mod autocomplete;
pub mod opensearch;
pub mod search;
pub mod settings;

use std::net::SocketAddr;

use axum::{http::header, routing::get, Router};

pub const BIND_ADDRESS: &str = "0.0.0.0:28019";

pub async fn run() {
    let app = Router::new()
        .route(
            "/",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                    include_str!("assets/index.html"),
                )
            }),
        )
        .route(
            "/settings.js",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
                    include_str!("assets/settings.js"),
                )
            }),
        )
        .route(
            "/style.css",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
                    include_str!("assets/style.css"),
                )
            }),
        )
        .route(
            "/script.js",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
                    include_str!("assets/script.js"),
                )
            }),
        )
        .route(
            "/robots.txt",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
                    include_str!("assets/robots.txt"),
                )
            }),
        )
        .route("/settings", get(settings::route))
        .route("/opensearch.xml", get(opensearch::route))
        .route("/search", get(search::route))
        .route("/autocomplete", get(autocomplete::route));

    println!("Listening on {BIND_ADDRESS}");

    let listener = tokio::net::TcpListener::bind(BIND_ADDRESS).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

pub fn get_blocked_domains<B>(cookies: &axum_extra::extract::cookie::CookieJar) -> B
where
    B: FromIterator<String> + Default,
{
    use base64::prelude::*;

    cookies
        .get("blocked")
        .map(|cookie| {
            let cookie_value = cookie.value();
            // 500 kb block list limit
            if cookie_value.len() > 500000 {
                return Default::default();
            }
            let blocked_domains_base64 = BASE64_STANDARD.decode(cookie_value).unwrap_or_default();
            let blocked_domains_str =
                std::str::from_utf8(&blocked_domains_base64).unwrap_or_default();
            blocked_domains_str
                .split(',')
                .map(|domain| domain.trim().to_string())
                .collect()
        })
        .unwrap_or_default()
}
