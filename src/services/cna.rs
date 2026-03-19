use rss::Channel;
use scraper::{Html, Selector};

use crate::utils::cna_model::NewsModel;

#[derive(Clone, Copy)]
pub enum NewsCategoryCNA {
    Latest,
    Asia,
    Business,
    Singapore,
    Sports,
    World,
    Today,
}

impl NewsCategoryCNA {
    pub const ALL: [NewsCategoryCNA; 7] = [
        NewsCategoryCNA::Latest,
        NewsCategoryCNA::Asia,
        NewsCategoryCNA::Business,
        NewsCategoryCNA::Singapore,
        NewsCategoryCNA::Sports,
        NewsCategoryCNA::World,
        NewsCategoryCNA::Today,
    ];
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
    pub async fn fetch_category(category: NewsCategoryCNA) -> String {
        reqwest::get(match category {
            NewsCategoryCNA::Latest => Self::LATEST_NEWS_URL,
            NewsCategoryCNA::Asia => Self::ASIA_URL,
            NewsCategoryCNA::Business => Self::BUSINESS_URL,
            NewsCategoryCNA::Singapore => Self::SINGAPORE_URL,
            NewsCategoryCNA::Sports => Self::SPORT_URL,
            NewsCategoryCNA::World => Self::WORLD_URL,
            NewsCategoryCNA::Today => Self::TODAY_URL,
        })
        .await
        .expect("Failed to fetch category")
        .text()
        .await
        .expect("Failed to get body of data")
    }
    pub async fn fetch_page(url: &String) -> String {
        reqwest::get(url)
            .await
            .expect("Failed to fetch url")
            .text()
            .await
            .expect("Failed to get body of data")
    }
    pub fn parse(xml_response: String) -> Vec<NewsModel> {
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
                let categories = cloned_item.categories;
                NewsModel {
                    title,
                    description,
                    content: None,
                    link,
                    pub_date,
                    categories: categories
                        .iter()
                        .map(|c| c.name.split(" ,").collect::<Vec<_>>().join(", "))
                        .collect(),
                }
            })
            .collect()
        // for (idx, cnamodel) in a.iter().enumerate() {
        //     println!("iter: {}", idx);
        //     println!("title: {}", cnamodel.title);
        //     println!("description: {}", cnamodel.description);
        //     println!("link: {}", cnamodel.link);
        //     println!("pub_date: {}", cnamodel.pub_date);
        //     for category in cnamodel.categories.iter() {
        //         println!("categories: {}", category);
        //     }
        // }
    }
    pub fn webscrape(xml_response: &String) -> Html {
        // gets content from link in CNAModel
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
