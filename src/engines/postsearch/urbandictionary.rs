use scraper::{Html, Selector};

use crate::engines::{answer::regex, Response, CLIENT};

pub fn request(response: &Response) -> Option<reqwest::RequestBuilder> {
    for search_result in response.search_results.iter().take(8) {
        if regex!(r"^https:\/\/www\.urbandictionary\.com\/define\.php\?term=[\w\.\-\+]+$")
            .is_match(&search_result.url)
        {
            log::info!("urban dictionary result: {}", search_result.url);
            return Some(CLIENT.get(search_result.url.as_str()));
        }
    }

    None
}

pub fn parse_response(body: &str) -> Option<String> {
    let dom = Html::parse_document(body);

    let answer_query = Selector::parse("div.definition > div").unwrap();
    let mut answers = dom.select(&answer_query);
    let answer = answers.next()?;

    let answer_title = answer
        .select(&Selector::parse("div.mb-8 > h1").unwrap())
        .next()?
        .html()
        .to_string();

    let answer_details_selector = Selector::parse("div.break-words").unwrap();
    let answer_details = answer.select(&answer_details_selector);
    let answer_details = answer_details
        .map(|details| details.html().to_string())
        .collect::<Vec<_>>()
        .join("<br />");

    let answer_html = format!(r#"{answer_title}<br />{answer_details}"#);

    let answer_html = ammonia::Builder::default()
        .url_relative(ammonia::UrlRelative::RewriteWithBase(
            "https://www.urbandictionary.com/".parse().unwrap(),
        ))
        .clean(&answer_html)
        .to_string();

    Some(answer_html)
}
