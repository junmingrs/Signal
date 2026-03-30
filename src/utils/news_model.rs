use crate::tui::tabs::news::NewsSource;

#[derive(Debug, Clone)]
pub struct NewsModel {
    pub title: String,
    pub description: String,
    pub content: Option<Vec<String>>,
    pub link: String,
    pub pub_date: String,
    pub categories: Vec<String>,
    pub source: NewsSource,
}
