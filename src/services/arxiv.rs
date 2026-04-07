use atom_syndication::Feed;

use crate::utils::{papers_model::PapersModel, time_formatter::rfc2822_to_custom};

pub struct Arxiv;

impl Arxiv {
    pub async fn fetch_rss() -> String {
        match reqwest::get("https://export.arxiv.org/api/query?search_query=cat:cs.SE&sortBy=submittedDate&sortOrder=descending&max_results=10").await {
            Ok(r) => { r.text().await.unwrap() }
            Err(_) => { String::new() }
        }
    }
    pub fn parse(xml_response: String) -> Vec<PapersModel> {
        if xml_response.is_empty() {
            return Vec::new();
        }
        let feed = xml_response.parse::<Feed>().unwrap();
        feed.entries
            .iter()
            .map(|entry| {
                let title = entry.title.clone().to_string();
                let summary = entry.summary.clone().unwrap().to_string();
                let fetched_pub_date = entry.published.unwrap().clone();
                let pub_date = rfc2822_to_custom(fetched_pub_date.to_rfc2822());
                let link = entry
                    .links
                    .iter()
                    .find(|l| l.rel.clone() == String::from("related"))
                    .unwrap()
                    .href
                    .clone();
                PapersModel {
                    title,
                    summary,
                    pub_date,
                    link,
                }
            })
            .collect()
    }
}
