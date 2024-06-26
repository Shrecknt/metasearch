use reqwest::Url;
use serde::Deserialize;

use crate::engines::{EngineResponse, EngineSearchResult, RequestResponse, CLIENT};

pub fn request(query: &str) -> RequestResponse {
    CLIENT
        .get(
            Url::parse_with_params(
                "https://api.yep.com/fs/2/search",
                &[
                    ("client", "web"),
                    ("gl", "all"),
                    ("no_correct", "true"),
                    ("q", query),
                    ("safeSearch", "off"),
                    ("type", "web"),
                ],
            )
            .unwrap(),
        )
        .into()
}

#[derive(Deserialize, Debug)]
struct YepApiResponse {
    pub results: Vec<YepApiResponseResult>,
}

#[derive(Deserialize, Debug)]
struct YepApiResponseResult {
    pub url: String,
    pub title: String,
    pub snippet: String,
}

pub fn parse_response(body: &str) -> eyre::Result<EngineResponse> {
    let (code, response): (String, YepApiResponse) = serde_json::from_str(body)?;
    if &code != "Ok" {
        return Ok(EngineResponse::new());
    }
    let search_results = response
        .results
        .iter()
        .map(|result| EngineSearchResult {
            url: result.url.clone(),
            title: result.title.clone(),
            description: result.snippet.clone(),
        })
        .collect();
    let engine_response = EngineResponse {
        search_results,
        featured_snippet: None,
        answer_html: None,
        infobox_html: None,
    };
    Ok(engine_response)
}
