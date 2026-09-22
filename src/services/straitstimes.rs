use std::fmt::{self};

use rss::Channel;
use scraper::{Html, Selector};

use crate::{
    tui::tabs::news::NewsSource,
    utils::{
        header::get_default_headers, article::Article, time_formatter::rfc2822_to_custom,
    },
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NewsCategoryST {
    Singapore(Option<Vec<Article>>),
    Asia(Option<Vec<Article>>),
    World(Option<Vec<Article>>),
    Opinion(Option<Vec<Article>>),
    Life(Option<Vec<Article>>),
    Business(Option<Vec<Article>>),
    Sport(Option<Vec<Article>>),
    Newsletter(Option<Vec<Article>>),
}

impl NewsCategoryST {
    pub const ALL: [NewsCategoryST; 8] = [
        NewsCategoryST::Singapore(None),
        NewsCategoryST::Asia(None),
        NewsCategoryST::World(None),
        NewsCategoryST::Opinion(None),
        NewsCategoryST::Life(None),
        NewsCategoryST::Business(None),
        NewsCategoryST::Sport(None),
        NewsCategoryST::Newsletter(None),
    ];
}

impl fmt::Display for NewsCategoryST {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            NewsCategoryST::Singapore(_) => "Singapore",
            NewsCategoryST::Asia(_) => "Asia",
            NewsCategoryST::World(_) => "World",
            NewsCategoryST::Opinion(_) => "Opinion",
            NewsCategoryST::Life(_) => "Life",
            NewsCategoryST::Business(_) => "Business",
            NewsCategoryST::Sport(_) => "Sport",
            NewsCategoryST::Newsletter(_) => "Newsletter",
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
        let headers = get_default_headers();

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        match client
            .get(match category {
                NewsCategoryST::Singapore(_) => Self::SINGAPORE_URL,
                NewsCategoryST::Asia(_) => Self::ASIA_URL,
                NewsCategoryST::World(_) => Self::WORLD_URL,
                NewsCategoryST::Opinion(_) => Self::OPINION_URL,
                NewsCategoryST::Life(_) => Self::LIFE_URL,
                NewsCategoryST::Business(_) => Self::BUSINESS_URL,
                NewsCategoryST::Sport(_) => Self::SPORT_URL,
                NewsCategoryST::Newsletter(_) => Self::NEWSLETTER_URL,
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
    pub async fn parse(xml_response: String, news_category: NewsCategoryST) -> Vec<Article> {
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
                source: NewsSource::StraitsTimes,
            });
        }
        news_models
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
