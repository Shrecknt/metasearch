use std::collections::{BTreeSet, HashMap};

use axum::{
    body::Body,
    extract::Query,
    http::{header, StatusCode},
    response::{IntoResponse, Redirect},
};
use html_escape::encode_single_quoted_attribute;

use super::{get_blocked_domains, set_blocked_domains};

pub async fn route(cookies: axum_extra::extract::cookie::CookieJar) -> impl IntoResponse {
    let blocked_domains = get_blocked_domains::<BTreeSet<_>>(&cookies);
    let sanitized_blocked_domains = blocked_domains
        .iter()
        .map(|domain| {
            let form_header = "<form action='/unblock_site' method='get' enctype='application/x-www-form-urlencoded' class='unblock-site-form'>";
            let text_display = format!(
                "<input type='text' value='{}' name='domain' contenteditable='false' />",
                encode_single_quoted_attribute(domain)
            );
            let return_elem = "<input type='text' name='return' value='/settings' style='display:none;'>";
            let button = "<input type='submit' value='x' />";
            let form_footer = "</form>";
            let form = format!("{form_header}{text_display}{return_elem}{button}{form_footer}");
            form
        })
        .collect::<Vec<_>>();

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        Body::from(
            include_str!("assets/settings.html")
                .replace("%blocked-sites%", &sanitized_blocked_domains.join("")),
        ),
    )
}

pub async fn block_route(
    Query(params): Query<HashMap<String, String>>,
    cookies: axum_extra::extract::cookie::CookieJar,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let Some(to_block) = params.get("domain") else {
        return Err((
            StatusCode::BAD_REQUEST,
            Body::from(format!("missing `domain` param\nparams:\n{params:?}")),
        ));
    };
    let Some(return_url) = params.get("return") else {
        return Err((
            StatusCode::BAD_REQUEST,
            Body::from(format!("missing `return` param\nparams:\n{params:?}")),
        ));
    };
    let mut blocked_domains = get_blocked_domains::<BTreeSet<_>>(&cookies);
    blocked_domains.insert(to_block.into());
    Ok((
        [(header::SET_COOKIE, set_blocked_domains(blocked_domains))],
        Redirect::to(return_url),
    ))
}

pub async fn unblock_route(
    Query(params): Query<HashMap<String, String>>,
    cookies: axum_extra::extract::cookie::CookieJar,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let Some(to_unblock) = params.get("domain") else {
        return Err((
            StatusCode::BAD_REQUEST,
            Body::from(format!("missing `domain` param\nparams:\n{params:?}")),
        ));
    };
    let Some(return_url) = params.get("return") else {
        return Err((
            StatusCode::BAD_REQUEST,
            Body::from(format!("missing `return` param\nparams:\n{params:?}")),
        ));
    };
    let mut blocked_domains = get_blocked_domains::<BTreeSet<_>>(&cookies);
    blocked_domains.remove(to_unblock);
    Ok((
        [(header::SET_COOKIE, set_blocked_domains(blocked_domains))],
        Redirect::to(return_url),
    ))
}
