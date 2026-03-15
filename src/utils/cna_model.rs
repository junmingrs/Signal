#[derive(Debug, Clone)]
pub struct CNAModel {
    pub title: String,
    pub description: String,
    pub content: Option<Vec<String>>,
    pub link: String,
    pub pub_date: String,
    pub categories: Vec<String>,
}
