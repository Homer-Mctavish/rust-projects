use lambda_runtime::Error;
use lambda_runtime::Context;
use serde_derive::{Serialize, Deserialize};

use chrono;
//regex used to select substrings from html elements
use regex::Regex;
//used for the HTTP get_client to obtain the HTML page
use reqwest::StatusCode;
//scrapes HTML elements from the wating HTML page asynchronously
use scraper::{Html, Selector};
use std::fs::File;
use std::io::Write;

mod utils;
mod models;

#[derive(Deserialize)]
struct Website {
    domain_name: String,
}


#[derive(Serialize)]
struct Output {
   scraped:Vec<models::ArticleData>,
}

fn save_raw_html(raw_html: &str, domain_name: &str) {
    let dt = chrono::Local::now();
    let filename = format!("{}_{}.html", domain_name, dt.format("%Y-%m-%d_%H.%M.%S"));
    let mut writer = File::create(&filename).unwrap();
    write!(&mut writer, "{}", &raw_html).unwrap();
}

fn save_article_list(article_list: &Vec<models::ArticleData>, domain_name: &str) {
    let dt = chrono::Local::now();
    let filename = format!("{}_{}.json", domain_name, dt.format("%Y-%m-%d_%H.%M.%S"));
    let mut writer = File::create(&filename).unwrap();
    write!(
        &mut writer,
        "{}",
        &serde_json::to_string(&article_list).unwrap()
    )
    .unwrap();
}

async fn handler(site: Website, context: Context) -> Result<Output, Error> {
    let client = utils::get_client();
    let domain_name = site.domain_name;
    let url = format!("https://{}", domain_name);
    let result = client.get(url).send().await.unwrap();
    let raw_html = match result.status() {
        StatusCode::OK => result.text().await.unwrap(),
        _ => panic!("Something went wrong"),
    };
    save_raw_html(&raw_html, &domain_name);
    let document = Html::parse_document(&raw_html);
    let article_selector = Selector::parse("a.js-content-viewer").unwrap();
    let h2select = Selector::parse("h2").unwrap();
    let h3select = Selector::parse("h3").unwrap();
    let get_text_re = Regex::new(r"->.*<").unwrap();
    let find_replace_re = Regex::new(r"[-><]").unwrap();
    let mut article_list: Vec<models::ArticleData> = Vec::new();
    for element in document.select(&article_selector) {
        let inner = element.inner_html().to_string();
        let mut h2el = element.select(&h2select);
        let mut h3el = element.select(&h3select);
        let href = match element.value().attr("href") {
            Some(target_url) => target_url,
            _ => "no url found",
        };
        match h2el.next() {
            Some(elref) => {
                let title = elref.inner_html().to_string();
                println!("H2: {}", &title);
                println!("Link: {}", &href);
                article_list.push(models::ArticleData {
                    article_title: title,
                    url_link: href.to_string(),
                    domain_name: domain_name.to_string(),
                });
                continue;
            }
            _ => {}
        }
        match h3el.next() {
            Some(elref) => {
                let title = elref.inner_html().to_string();
                println!("H3: {}", &title);
                println!("Link: {}", &href);
                article_list.push(models::ArticleData {
                    article_title: title,
                    url_link: href.to_string(),
                    domain_name: domain_name.to_string(),
                });
                continue;
            }
            _ => {}
        }

        match get_text_re.captures_iter(&inner).next() {
            Some(cap) => {
                let replaced = find_replace_re.replace_all(&cap[0], "");
                println!("Regex: {}", &replaced);
                println!("Link: {}", &href);
                article_list.push(models::ArticleData {
                    article_title: replaced.to_string(),
                    url_link: href.to_string(),
                    domain_name: domain_name.to_string(),
                });
            }
            _ => {
                println!("Nothing found");
            }
        }
    }
    Ok(Output {
        scraped: article_list,
    })
}

//we want the main from webscraper to fill struct sitelist with 
//the results of our webscraping


#[tokio::main]
async fn main() -> Result<(), Error> {
    let handler = lambda_runtime::handler_fn(handler);
    lambda_runtime::run(handler).await?;
    Ok(())
}
