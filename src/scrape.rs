// src/scrape.rs

use reqwest::Client;
use crate::dncli::DncliError;
use html5ever::{parse_document, tendril::TendrilSink};
use scraper::{Html, Selector};
use std::collections::HashSet;
use url::Url;

#[derive(Debug)]
pub enum ScrapeType {
    Html,
    Css,
    JavaScript,
}

pub async fn scrape_url(url: &str, scrape_type: ScrapeType) -> Result<String, DncliError> {
    let client = Client::new();
    println!("Starting {} scraping for: {}", format!("{:?}", scrape_type).to_lowercase(), url);

    let mut parsed_url = Url::parse(url).map_err(DncliError::UrlParse)?;
    if parsed_url.scheme().is_empty() {
        let new_url_str = format!("https://{}", url);
        parsed_url = Url::parse(&new_url_str).map_err(|e| DncliError::UrlParse(e))?;
    }
    let final_url = parsed_url.to_string();

    let response = client.get(&final_url).send().await?.error_for_status()?;
    let content = response.text().await?;

    match scrape_type {
        ScrapeType::Html => {
            let document = parse_document(TendrilSink::default(), Default::default())
                .from_utf8()
                .read_from(&mut content.as_bytes())
                .unwrap();
            
            println!("HTML content fetched and parsed for: {}", final_url);
            Ok(document.to_string())
        }
        ScrapeType::Css => {
            let document = Html::parse(&content);
            let css_selectors = Selector::parse("style, link[rel='stylesheet'][href]").unwrap();
            let mut css_content = String::new();
            let mut base_url = Url::parse(&final_url).map_err(DncliError::UrlParse)?;

            base_url.set_fragment(None);

            let mut fetched_css_urls = HashSet::new();

            for element in document.select(&css_selectors) {
                if element.tag().name() == "style" {
                    css_content.push_str(element.text().collect::<String>().as_str());
                    css_content.push_str("\n\n/* --- End of inline style --- */\n\n");
                } else if element.tag().name() == "link" {
                    if let Some(href) = element.value().attr("href") {
                        let absolute_url = base_url.join(href)
                            .map_err(DncliError::UrlParse)?;
                        let css_url = absolute_url.to_string();

                        if !fetched_css_urls.contains(&css_url) {
                            println!("Fetching linked CSS from: {}", css_url);
                            match client.get(&css_url).send().await {
                                Ok(css_response) => {
                                    if css_response.status().is_success() {
                                        let linked_css = css_response.text().await?;
                                        css_content.push_str(&format!("\n\n/* --- Linked CSS from {} --- */\n", css_url));
                                        css_content.push_str(&linked_css);
                                        css_content.push_str("\n\n");
                                        fetched_css_urls.insert(css_url);
                                    } else {
                                        eprintln!("Failed to fetch linked CSS from {}: Status {}", css_url, css_response.status());
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Error fetching linked CSS from {}: {}", css_url, e);
                                }
                            }
                        }
                    }
                }
            }
            if css_content.is_empty() {
                Err(DncliError::Other(format!("No CSS content found or linked at {}", final_url)))
            } else {
                println!("CSS content extracted for: {}", final_url);
                Ok(css_content)
            }
        }
        ScrapeType::JavaScript => {
            let document = Html::parse(&content);
            let js_selectors = Selector::parse("script[src], script:not([src])").unwrap();
            let mut js_content = String::new();
            let mut base_url = Url::parse(&final_url).map_err(DncliError::UrlParse)?;

            base_url.set_fragment(None);

            let mut fetched_js_urls = HashSet::new();

            for element in document.select(&js_selectors) {
                if let Some(src) = element.value().attr("src") {
                    let absolute_url = base_url.join(src)
                        .map_err(DncliError::UrlParse)?;
                    let js_url = absolute_url.to_string();

                    if !fetched_js_urls.contains(&js_url) {
                        println!("Fetching linked JavaScript from: {}", js_url);
                        match client.get(&js_url).send().await {
                            Ok(js_response) => {
                                if js_response.status().is_success() {
                                    let linked_js = js_response.text().await?;
                                    js_content.push_str(&format!("\n\n/* --- Linked JavaScript from {} --- */\n", js_url));
                                    js_content.push_str(&linked_js);
                                    js_content.push_str("\n\n");
                                    fetched_js_urls.insert(js_url);
                                } else {
                                    eprintln!("Failed to fetch linked JavaScript from {}: Status {}", js_url, js_response.status());
                                }
                            }
                            Err(e) => {
                                eprintln!("Error fetching linked JavaScript from {}: {}", js_url, e);
                            }
                        }
                    }
                } else {
                    js_content.push_str(element.text().collect::<String>().as_str());
                    js_content.push_str("\n\n/* --- End of inline script --- */\n\n");
                }
            }
            if js_content.is_empty() {
                Err(DncliError::Other(format!("No JavaScript content found or linked at {}", final_url)))
            } else {
                println!("JavaScript content extracted for: {}", final_url);
                Ok(js_content)
            }
        }
    }
}

