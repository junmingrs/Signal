use std::fmt::{self};

use rss::Channel;
use scraper::{Html, Selector};

use crate::{
    tui::tabs::news::NewsSource,
    utils::{article::Article, header::get_default_headers, time_formatter::rfc2822_to_custom},
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NewsCategoryBT {
    Singapore(Option<Vec<Article>>),
    International(Option<Vec<Article>>),
    Opinion(Option<Vec<Article>>),
    Market(Option<Vec<Article>>),
    Technology(Option<Vec<Article>>),
    Awards(Option<Vec<Article>>),
}

impl NewsCategoryBT {
    pub const ALL: [NewsCategoryBT; 6] = [
        NewsCategoryBT::Singapore(None),
        NewsCategoryBT::International(None),
        NewsCategoryBT::Opinion(None),
        NewsCategoryBT::Market(None),
        NewsCategoryBT::Technology(None),
        NewsCategoryBT::Awards(None),
    ];
}

impl fmt::Display for NewsCategoryBT {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            NewsCategoryBT::Singapore(_) => "Singapore",
            NewsCategoryBT::International(_) => "International",
            NewsCategoryBT::Opinion(_) => "Opinion",
            NewsCategoryBT::Market(_) => "Market",
            NewsCategoryBT::Technology(_) => "Technology",
            NewsCategoryBT::Awards(_) => "Awards",
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
        let headers = get_default_headers();

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        match client
            .get(match category {
                NewsCategoryBT::Singapore(_) => Self::SINGAPORE_URL,
                NewsCategoryBT::International(_) => Self::INTERNATIONAL_URL,
                NewsCategoryBT::Opinion(_) => Self::OPINION_URL,
                NewsCategoryBT::Market(_) => Self::MARKET_URL,
                NewsCategoryBT::Technology(_) => Self::TECHNOLOGY_URL,
                NewsCategoryBT::Awards(_) => Self::AWARDS_URL,
            })
            .send()
            .await
        {
            Ok(r) => r.text().await.unwrap(),
            Err(_) => String::new(),
        }
    }
    pub async fn fetch_page(url: &String) -> String {
        let headers = get_default_headers();

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
    pub async fn parse(xml_response: String, news_category: NewsCategoryBT) -> Vec<Article> {
        if xml_response.is_empty() {
            return Vec::new();
        }

        let mut news_models = Vec::new();
        let channel = Channel::read_from(xml_response.as_bytes()).unwrap();
        for item in channel.items {
            let cloned_item = item.clone();
            let title = cloned_item.title.unwrap_or("".to_string());
            let description = cloned_item.description.unwrap_or("".to_string());
            let link = cloned_item.link.unwrap_or("".to_string());
            let pub_date = cloned_item.pub_date.unwrap_or("".to_string());
            let formatted_pub_date = rfc2822_to_custom(pub_date);
            let page = Self::fetch_page(&link).await;
            let document = Self::webscrape(&page);
            let content = Self::get_content(document);
            news_models.push(Article {
                title,
                description,
                content,
                link,
                pub_date: formatted_pub_date,
                categories: vec![news_category.to_string()],
                source: NewsSource::BusinessTimes,
            });
        }
        news_models
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
