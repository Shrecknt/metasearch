pub mod autocomplete;
pub mod opensearch;
pub mod search;
pub mod settings;

use std::{collections::BTreeMap, net::SocketAddr};

use axum::{http::header, routing::get, Router};

use crate::engines::Engine;

pub const BIND_ADDRESS: &str = "0.0.0.0:28019";

pub const DISALLOWED_CHARACTERS: &[char] = &[
    '\u{0007}', '\u{0008}', '\u{001b}', '\u{000c}', '\u{000a}', '\u{000d}', '\u{0009}', '\u{000b}',
];

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
        .route(
            "/icons/graduation_cap.svg",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "image/svg+xml")],
                    include_str!("assets/icons/graduation_cap.svg"),
                )
            }),
        )
        .route(
            "/icons/gear.svg",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "image/svg+xml")],
                    include_str!("assets/icons/gear.svg"),
                )
            }),
        )
        .route(
            "/icons/house.svg",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "image/svg+xml")],
                    include_str!("assets/icons/house.svg"),
                )
            }),
        )
        .route("/settings", get(settings::route))
        .route("/block_site", get(settings::block_route))
        .route("/unblock_site", get(settings::unblock_route))
        .route("/set_search_engines", get(settings::search_engines_route))
        .route("/rand_noscript", get(crate::engines::answer::random::route))
        .route("/opensearch.xml", get(opensearch::route))
        .route("/search", get(search::route))
        .route("/autocomplete", get(autocomplete::route));

    log::info!("Listening on {BIND_ADDRESS}");

    let listener = tokio::net::TcpListener::bind(BIND_ADDRESS).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

pub fn get_enabled_search_engines(
    cookies: &axum_extra::extract::cookie::CookieJar,
) -> BTreeMap<String, bool> {
    let engines = cookies
        .get("engines")
        .map(|cookie| {
            let cookie_value = cookie.value();
            let engines_base64 = {
                use base64::prelude::*;
                BASE64_STANDARD.decode(cookie_value).unwrap_or_default()
            };
            let engines_str = std::str::from_utf8(&engines_base64).unwrap_or_default();
            engines_str
                .split(',')
                .map(|entry| entry.trim())
                .filter(|entry| !entry.is_empty())
                .map(|line| {
                    let mut split = line.split('=');
                    let key = split.next().unwrap_or_default();
                    let value = split.next().unwrap_or_default() == "true";
                    (key.to_string(), value)
                })
                .collect::<BTreeMap<String, bool>>()
        })
        .unwrap_or_default();
    let mut res = BTreeMap::new();
    for engine in Engine::all() {
        let id = engine.id();
        res.insert(
            id.to_string(),
            *engines.get(id).unwrap_or(&engine.is_enabled_by_default()),
        );
    }
    res
}

pub fn set_enabled_search_engines(enabled_search_engines: &BTreeMap<String, bool>) -> String {
    let mut first_iter = true;
    let mut built_string = String::new();
    for (engine, enabled) in enabled_search_engines.into_iter() {
        if first_iter {
            first_iter = false;
        } else {
            built_string.push(',');
        }
        built_string.push_str(&format!("{engine}={enabled}"));
    }
    use base64::prelude::*;
    let blocked_domains_base64 = BASE64_STANDARD.encode(built_string);
    format!("engines={blocked_domains_base64}")
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
                .filter(|domain| !domain.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

pub fn set_blocked_domains<B>(blocked_domains: B) -> String
where
    B: IntoIterator<Item = String>,
{
    let mut first_iter = true;
    let mut built_string = String::new();
    for domain in blocked_domains.into_iter() {
        if first_iter {
            first_iter = false;
        } else {
            built_string.push(',');
        }
        built_string.push_str(&domain);
    }
    use base64::prelude::*;
    let blocked_domains_base64 = BASE64_STANDARD.encode(built_string);
    format!("blocked={blocked_domains_base64}")
}
