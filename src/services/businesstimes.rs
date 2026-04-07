use std::fmt::{self};

use reqwest::header::{ACCEPT, HeaderMap, HeaderValue, USER_AGENT};
use rss::Channel;
use scraper::{Html, Selector};

use crate::{
    tui::tabs::news::NewsSource,
    utils::{news_model::NewsModel, time_formatter::rfc2822_to_custom},
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NewsCategoryBT {
    Singapore,
    International,
    Opinion,
    Market,
    Technology,
    Awards,
}

impl NewsCategoryBT {
    pub const ALL: [NewsCategoryBT; 6] = [
        NewsCategoryBT::Singapore,
        NewsCategoryBT::International,
        NewsCategoryBT::Opinion,
        NewsCategoryBT::Market,
        NewsCategoryBT::Technology,
        NewsCategoryBT::Awards,
    ];
}

impl fmt::Display for NewsCategoryBT {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            NewsCategoryBT::Singapore => "Singapore",
            NewsCategoryBT::International => "International",
            NewsCategoryBT::Opinion => "Opinion",
            NewsCategoryBT::Market => "Market",
            NewsCategoryBT::Technology => "Technology",
            NewsCategoryBT::Awards => "Awards",
        };
        write!(f, "{}", s)
    }
}

pub struct BT;

impl BT {
    const SINGAPORE_URL: &str = "https://www.businesstimes.com.sg/rss/singapore";
    const INTERNATIONAL_URL: &str = "https://www.businesstimes.com.sg/rss/international";
    const OPINION_URL: &str = "https://www.businesstimes.com.sg/rss/opinion-features";
    const MARKET_URL: &str = "https://www.businesstimes.com.sg/rss/companies-markets";
    const TECHNOLOGY_URL: &str = "https://www.businesstimes.com.sg/rss/startups-tech";
    const AWARDS_URL: &str = "https://www.businesstimes.com.sg/rss/events-awards";
    pub async fn fetch_category(category: &NewsCategoryBT) -> String {
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
                NewsCategoryBT::Singapore => Self::SINGAPORE_URL,
                NewsCategoryBT::International => Self::INTERNATIONAL_URL,
                NewsCategoryBT::Opinion => Self::OPINION_URL,
                NewsCategoryBT::Market => Self::MARKET_URL,
                NewsCategoryBT::Technology => Self::TECHNOLOGY_URL,
                NewsCategoryBT::Awards => Self::AWARDS_URL,
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
    pub fn parse(xml_response: String, news_category: NewsCategoryBT) -> Vec<NewsModel> {
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
                    source: NewsSource::BusinessTimes,
                }
            })
            .collect()
    }
    pub fn webscrape(xml_response: &String) -> Html {
        Html::parse_document(xml_response)
    }
    pub fn get_content(document: Html) -> Vec<String> {
        let selector = Selector::parse(r#"div[data-testid="article-body-container"] p"#).unwrap();
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
