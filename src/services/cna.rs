use std::fmt::{self};

use rss::Channel;
use scraper::{Html, Selector};

use crate::{
    tui::tabs::news::{Article, NewsSource},
    utils::{article::Article, time_formatter::rfc2822_to_custom},
};

#[derive(Copy, PartialEq, Debug)]
pub enum NewsCategoryCNA {
    Latest(Option<Vec<Article>>),
    Asia(Option<Vec<Article>>),
    Business(Option<Vec<Article>>),
    Singapore(Option<Vec<Article>>),
    Sports(Option<Vec<Article>>),
    World(Option<Vec<Article>>),
    Today(Option<Vec<Article>>),
}

impl NewsCategoryCNA {
    pub const ALL: [NewsCategoryCNA; 7] = [
        NewsCategoryCNA::Latest(None),
        NewsCategoryCNA::Asia(None),
        NewsCategoryCNA::Business(None),
        NewsCategoryCNA::Singapore(None),
        NewsCategoryCNA::Sports(None),
        NewsCategoryCNA::World(None),
        NewsCategoryCNA::Today(None),
    ];

    pub fn update_articles(&mut self, articles: Vec<Article>) {
        match self {
            NewsCategoryCNA::Latest(articles_opt)
            | NewsCategoryCNA::Asia(articles_opt)
            | NewsCategoryCNA::Business(articles_opt)
            | NewsCategoryCNA::Singapore(articles_opt)
            | NewsCategoryCNA::Sports(articles_opt)
            | NewsCategoryCNA::World(articles_opt)
            | NewsCategoryCNA::Today(articles_opt) => {
                *articles_opt = Some(articles);
            }
        }
    }
}

impl fmt::Display for NewsCategoryCNA {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            NewsCategoryCNA::Latest(_) => "Latest",
            NewsCategoryCNA::Asia(_) => "Asia",
            NewsCategoryCNA::Business(_) => "Business",
            NewsCategoryCNA::Singapore(_) => "Singapore",
            NewsCategoryCNA::Sports(_) => "Sports",
            NewsCategoryCNA::World(_) => "World",
            NewsCategoryCNA::Today(_) => "Today",
        };
        write!(f, "{}", s)
    }
}

pub struct CNA;

impl CNA {
    const LATEST_NEWS_URL: &str =
        "https://www.channelnewsasia.com/api/v1/rss-outbound-feed?_format=xml";
    const ASIA_URL: &str =
        "https://www.channelnewsasia.com/api/v1/rss-outbound-feed?_format=xml&category=6511";
    const BUSINESS_URL: &str =
        "https://www.channelnewsasia.com/api/v1/rss-outbound-feed?_format=xml&category=6936";
    const SINGAPORE_URL: &str =
        "https://www.channelnewsasia.com/api/v1/rss-outbound-feed?_format=xml&category=10416";
    const SPORT_URL: &str =
        "https://www.channelnewsasia.com/api/v1/rss-outbound-feed?_format=xml&category=10296";
    const WORLD_URL: &str =
        "https://www.channelnewsasia.com/api/v1/rss-outbound-feed?_format=xml&category=6311";
    const TODAY_URL: &str =
        "https://www.channelnewsasia.com/api/v1/rss-outbound-feed?_format=xml&category=679471";
    pub async fn fetch_category(category: &NewsCategoryCNA) -> String {
        match reqwest::get(match category {
            NewsCategoryCNA::Latest(_) => Self::LATEST_NEWS_URL,
            NewsCategoryCNA::Asia(_) => Self::ASIA_URL,
            NewsCategoryCNA::Business(_) => Self::BUSINESS_URL,
            NewsCategoryCNA::Singapore(_) => Self::SINGAPORE_URL,
            NewsCategoryCNA::Sports(_) => Self::SPORT_URL,
            NewsCategoryCNA::World(_) => Self::WORLD_URL,
            NewsCategoryCNA::Today(_) => Self::TODAY_URL,
        })
        .await
        {
            Ok(r) => r.text().await.unwrap(),
            Err(_) => String::new(),
        }
    }
    pub async fn fetch_page(url: &String) -> String {
        reqwest::get(url)
            .await
            .expect("Failed to fetch url")
            .text()
            .await
            .expect("Failed to get body of data")
    }
    pub async fn parse(xml_response: String) -> Vec<Article> {
        if xml_response.is_empty() {
            return Vec::new();
        }

        let mut news_models = Vec::new();
        let channel = Channel::read_from(xml_response.as_bytes()).unwrap();
        for item in channel.items {
            let title = item.title.unwrap_or("".to_string());
            let description = item.description.unwrap_or("".to_string());
            let link = item.link.unwrap_or("".to_string());
            let pub_date = item.pub_date.unwrap_or("".to_string());
            let formatted_pub_date = rfc2822_to_custom(pub_date);
            let categories = item
                .categories
                .iter()
                .map(|c| c.name.split(" ,").collect::<Vec<_>>().join(", "))
                .collect();

            let page = Self::fetch_page(&link).await;
            let document = Self::webscrape(&page);
            let content = Self::get_content(document);
            news_models.push(Article {
                title,
                description,
                content,
                link,
                pub_date: formatted_pub_date,
                categories,
                source: NewsSource::CNA,
            });
        }
        news_models
    }
    pub fn webscrape(xml_response: &String) -> Html {
        Html::parse_document(xml_response)
    }
    pub fn get_content(document: Html) -> Vec<String> {
        let selector = Selector::parse(r#"section[data-title="Content"] div.text-long p"#).unwrap();
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
