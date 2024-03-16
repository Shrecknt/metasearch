use axum::{
    body::Body,
    http::{header, StatusCode},
    response::IntoResponse,
};
use html_escape::encode_single_quoted_attribute;

use super::get_blocked_domains;

pub async fn route(cookies: axum_extra::extract::cookie::CookieJar) -> impl IntoResponse {
    let blocked_domains = get_blocked_domains::<Vec<_>>(&cookies);
    let sanitized_blocked_domains = blocked_domains
        .iter()
        .map(|domain| {
            let text_display = format!(
                "<input type='text' value='{}' name='domain' disabled />",
                encode_single_quoted_attribute(domain)
            );
            let button = "<input type='submit' value='x' />";
            let form = format!("{text_display}{button}");
            form
        })
        .collect::<Vec<_>>();

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        Body::from(
            include_str!("assets/settings.html")
                .replace("%blocked-sites%", &sanitized_blocked_domains.join("<br />")),
        ),
    )
}
