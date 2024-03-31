use std::collections::HashMap;

use axum::{extract::Query, http::header, response::IntoResponse};
use rand::Rng;

use crate::engines::{EngineResponse, SearchQuery};

use super::regex;

pub fn request(query: &SearchQuery) -> EngineResponse {
    if !regex!("^(rng)|(rand(om)?( number( generator)?)?)").is_match(&query.query.to_lowercase()) {
        return EngineResponse::new();
    }

    EngineResponse::answer_html(format!(
        r#"<h3>random number generator</h3>
<div id="random-container">
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
</div>"#
    ))
}

pub async fn route(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let min = params
        .get("min")
        .map(|v| v.parse::<i64>().ok())
        .flatten()
        .unwrap_or(1);
    let max = params
        .get("max")
        .map(|v| v.parse::<i64>().ok())
        .flatten()
        .unwrap_or(10);

    const METASEARCH_PREMIUM_PROMPT: &str = "<span style='font-size:0.25em;line-height:1.5em;display:block;'>metasearch premium allows generating random numbers up to 20 digits!</span>";

    let in_bounds = min >= -9999999999 && max <= 9999999999;

    let value = if in_bounds {
        rand::thread_rng()
            .gen_range(
                min..={
                    if min > max {
                        min
                    } else {
                        max
                    }
                },
            )
            .to_string()
    } else {
        METASEARCH_PREMIUM_PROMPT.to_string()
    };

    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        format!(
            r#"<!DOCTYPE html>
<html>
    <head>
        <link rel="stylesheet" href="/style.css">
    </head>
    <body style="background-color:#0d1017;">
        <table style="table-layout:fixed;width:100%;text-align:center;">
            <tr>
                <td id="output" style="font-size:4em;">
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
    <script>
        const randForm = document.forms[0];
        const outputElem = document.getElementById("output");

        function random(min, max) {{
            return Math.floor((Math.random() * (max - min + 1)) + min);
        }}

        randForm.onsubmit = (event) => {{
            event.preventDefault();
            const values = [...randForm.getElementsByTagName("input")]
                .filter(field => field.name !== "")
                .reduce((initial, field) => ({{
                    ...initial,
                    [field.name]: field
                }}), {{}});
            const min = Math.floor(Number(values.min.value ?? 1) ?? 1);
            const max = Math.floor(Number(values.max.value ?? 10) ?? 10);
            values.min.value = min;
            values.max.value = max;
            if (min > max) {{
                outputElem.innerHTML = "error: min > max";
                return;
            }}
            if (max > 9999999999 || min < -9999999999) {{
                outputElem.innerHTML = "{METASEARCH_PREMIUM_PROMPT}";
                return;
            }}
            const number = random(min, max);
            outputElem.innerHTML = number;
        }};
    </script>
</html>"#
        ),
    )
}
