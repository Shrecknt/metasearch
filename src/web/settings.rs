use std::collections::{BTreeMap, BTreeSet, HashMap};

use axum::{
    body::Body,
    extract::Query,
    http::{header, StatusCode},
    response::{IntoResponse, Redirect},
};
use html_escape::encode_single_quoted_attribute;

use crate::engines::Engine;

use super::{
    get_blocked_domains, get_enabled_search_engines, set_blocked_domains,
    set_enabled_search_engines,
};

pub async fn route(cookies: axum_extra::extract::cookie::CookieJar) -> impl IntoResponse {
    let enabled_search_engines = get_enabled_search_engines(&cookies);
    let search_engines = enabled_search_engines
        .iter()
        .map(|(engine, enabled)| {
            let engine_proper = match Engine::from_id(&engine) {
                Some(engine) => engine.id_proper(),
                None => "",
            };
            let is_checked = if *enabled { "checked" } else { "" };
            let checkbox =
                format!("<input type='checkbox' id='{engine}' name='{engine}' {is_checked} />");
            let name_label = format!("<label for='{engine}'>{engine_proper}</label>");
            format!("<tr><td class='engine-checkbox'>{checkbox}</td><td class='engine-label'>{name_label}</td></tr>")
        })
        .collect::<Vec<_>>();

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
                .replace("%search-engines%", &search_engines.join(""))
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
            [(header::CONTENT_TYPE, "text/plain")],
            Body::from(format!("missing `domain` param\nparams:\n{params:?}")),
        ));
    };
    let Some(return_url) = params.get("return") else {
        return Err((
            StatusCode::BAD_REQUEST,
            [(header::CONTENT_TYPE, "text/plain")],
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
            [(header::CONTENT_TYPE, "text/plain")],
            Body::from(format!("missing `domain` param\nparams:\n{params:?}")),
        ));
    };
    let Some(return_url) = params.get("return") else {
        return Err((
            StatusCode::BAD_REQUEST,
            [(header::CONTENT_TYPE, "text/plain")],
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

pub async fn search_engines_route(
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let Some(return_url) = params.get("return") else {
        return Err((
            StatusCode::BAD_REQUEST,
            [(header::CONTENT_TYPE, "text/plain")],
            Body::from(format!("missing `return` param\nparams:\n{params:?}")),
        ));
    };
    let mut enabled_search_engines = Engine::all()
        .iter()
        .map(|engine| (engine.id().to_string(), false))
        .collect::<BTreeMap<_, _>>();
    for (param, enabled) in &params {
        let enabled = enabled == "on";
        if let Some(value) = enabled_search_engines.get_mut(param) {
            *value = enabled;
        }
    }
    Ok((
        [(
            header::SET_COOKIE,
            set_enabled_search_engines(&enabled_search_engines),
        )],
        Redirect::to(return_url),
    ))
}
