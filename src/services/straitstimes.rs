use std::fmt::{self};

use reqwest::header::{ACCEPT, HeaderMap, HeaderValue, USER_AGENT};
use rss::Channel;
use scraper::{Html, Selector};

use crate::{
    tui::tabs::news::NewsSource,
    utils::{news_model::NewsModel, time_formatter::rfc2822_to_custom},
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NewsCategoryST {
    Singapore,
    Asia,
    World,
    Opinion,
    Life,
    Business,
    Sport,
    Newsletter,
}

impl NewsCategoryST {
    pub const ALL: [NewsCategoryST; 8] = [
        NewsCategoryST::Singapore,
        NewsCategoryST::Asia,
        NewsCategoryST::World,
        NewsCategoryST::Opinion,
        NewsCategoryST::Life,
        NewsCategoryST::Business,
        NewsCategoryST::Sport,
        NewsCategoryST::Newsletter,
    ];
}

impl fmt::Display for NewsCategoryST {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            NewsCategoryST::Singapore => "Singapore",
            NewsCategoryST::Asia => "Asia",
            NewsCategoryST::World => "World",
            NewsCategoryST::Opinion => "Opinion",
            NewsCategoryST::Life => "Life",
            NewsCategoryST::Business => "Business",
            NewsCategoryST::Sport => "Sport",
            NewsCategoryST::Newsletter => "Newsletter",
        };
        write!(f, "{}", s)
    }
}

pub struct ST;

impl ST {
    const SINGAPORE_URL: &str = "https://www.straitstimes.com/news/singapore/rss.xml";
    const ASIA_URL: &str = "https://www.straitstimes.com/news/asia/rss.xml";
    const WORLD_URL: &str = "https://www.straitstimes.com/news/world/rss.xml";
    const OPINION_URL: &str = "https://www.straitstimes.com/news/opinion/rss.xml";
    const LIFE_URL: &str = "https://www.straitstimes.com/news/life/rss.xml";
    const BUSINESS_URL: &str = "https://www.straitstimes.com/news/business/rss.xml";
    const SPORT_URL: &str = "https://www.straitstimes.com/news/sport/rss.xml";
    const NEWSLETTER_URL: &str = "https://www.straitstimes.com/news/newsletter/rss.xml";
    pub async fn fetch_category(category: &NewsCategoryST) -> String {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/120.0 Safari/537.36",
            ),
        );
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/rss+xml, application/xml;q=0.9, */*;q=0.8"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        match client
            .get(match category {
                NewsCategoryST::Singapore => Self::SINGAPORE_URL,
                NewsCategoryST::Asia => Self::ASIA_URL,
                NewsCategoryST::World => Self::WORLD_URL,
                NewsCategoryST::Opinion => Self::OPINION_URL,
                NewsCategoryST::Life => Self::LIFE_URL,
                NewsCategoryST::Business => Self::BUSINESS_URL,
                NewsCategoryST::Sport => Self::SPORT_URL,
                NewsCategoryST::Newsletter => Self::NEWSLETTER_URL,
            })
            .send()
            .await
        {
            Ok(r) => r.text().await.unwrap(),
            Err(_) => String::new(),
        }
    }
    pub async fn fetch_page(url: &String) -> String {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/120.0 Safari/537.36",
            ),
        );
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/rss+xml, application/xml;q=0.9, */*;q=0.8"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();
        let res = client
            .get(url)
            .send()
            .await
            .expect("Failed to get body of data");
        res.text().await.unwrap()
    }
    pub fn parse(xml_response: String, news_category: NewsCategoryST) -> Vec<NewsModel> {
        if xml_response.is_empty() {
            return Vec::new();
        }
        let channel = Channel::read_from(xml_response.as_bytes()).unwrap();
        channel
            .items
            .iter()
            .map(|item| {
                let cloned_item = item.clone();
                let title = cloned_item.title.unwrap_or("".to_string());
                let description = cloned_item.description.unwrap_or("".to_string());
                let link = cloned_item.link.unwrap_or("".to_string());
                let pub_date = cloned_item.pub_date.unwrap_or("".to_string());
                let formatted_pub_date = rfc2822_to_custom(pub_date);
                NewsModel {
                    title,
                    description,
                    content: None,
                    link,
                    pub_date: formatted_pub_date,
                    categories: vec![news_category.to_string()],
                    source: NewsSource::StraitsTimes,
                }
            })
            .collect()
    }
    pub fn webscrape(xml_response: &String) -> Html {
        Html::parse_document(xml_response)
    }
    pub fn get_content(document: Html) -> Vec<String> {
        let selector = Selector::parse(r#"div.storyline-wrapper p"#).unwrap();
        document
            .select(&selector)
            .filter_map(|el| {
                let text: String = el.text().collect();
                if text.trim().is_empty() {
                    None
                } else {
                    Some(text)
                }
            })
            .collect::<Vec<_>>()
    }
}
