use std::collections::HashMap;

use axum::{extract::Query, http::header, response::IntoResponse};
use rand::Rng;

use crate::engines::{EngineResponse, SearchQuery};

use super::regex;

pub fn request(query: &SearchQuery) -> EngineResponse {
    if !regex!("^random").is_match(&query.query.to_lowercase()) {
        return EngineResponse::new();
    }

    EngineResponse::answer_html(format!(
        r#"<h3>random</h3>
<div id="random-container">
    <noscript>
        <style>
            #random-iframe {{
                width: calc( 100% - 16px );
                height: calc( 100% - 16px );
                margin: 8px;
                border: none;
                outline: none;
            }}
        </style>
        <iframe id="random-iframe" src="/rand_noscript"></iframe>
    </noscript>
</div>
<script>
    const randomEl = document.getElementById("random-container");
</script>"#
    ))
}

pub async fn route(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let min = params
        .get("min")
        .map(|v| v.parse::<i32>().ok())
        .flatten()
        .unwrap_or(1);
    let max = params
        .get("max")
        .map(|v| v.parse::<i32>().ok())
        .flatten()
        .unwrap_or(10);

    let value = rand::thread_rng().gen_range(
        min..={
            if min > max {
                min
            } else {
                max
            }
        },
    );

    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        format!(
            r#"<!DOCTYPE html>
<html>
    <head>
        <link rel="stylesheet" href="/style.css">
    </head>
    <body style="background-color:#0d1017;">
        min = {min}, max = {max}<br />
        value = {value}
        <table style="table-layout:fixed;width:100%;text-align:center;">
            <tr>
                <td>
                    {value}
                </td>
                <td>
                    <form action="/rand_noscript" method="get" enctype="application/x-www-form-urlencoded">
                        <label for="min">Min</label><br /><input type="number" id="min" name="min" value="{min}"><br /><br />
                        <label for="max">Max</label><br /><input type="number" id="max" name="max" value="{max}"><br /><br />
                        <input type="submit" value="Generate">
                    </form>
                </td>
            </tr>
        </table>
    </body>
</html>"#
        ),
    )
}
