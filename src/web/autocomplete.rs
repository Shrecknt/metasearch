use std::{collections::HashMap, net::SocketAddr};

use axum::{
    extract::{ConnectInfo, Query},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};

use crate::engines;

pub async fn route(
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let ip = headers
        .get(std::env::var("IP_HEADER").unwrap_or("x-forwarded-for".into()))
        .map(|ip| ip.to_str().unwrap_or_default().to_string())
        .unwrap_or_else(|| addr.ip().to_string());

    let query = params
        .get("q")
        .cloned()
        .unwrap_or_default()
        .replace('\n', " ");

    if query.contains('\u{001b}') {
        return (
            StatusCode::BAD_REQUEST,
            Json((query.replace('\u{001b}', ""), vec![])),
        );
    }

    log::info!("Autocomplete request from {ip} for '{query}'");

    let query = if rustrict::CensorStr::is_inappropriate(query.as_str()) {
        rustrict::CensorStr::censor(query.as_str())
    } else {
        query
    };

    let res = match engines::autocomplete(&query).await {
        Ok(res) => res,
        Err(err) => {
            log::error!("Autocomplete error for '{query}': {}", err);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json((query, vec![])));
        }
    };

    (StatusCode::OK, Json((query, res)))
}
