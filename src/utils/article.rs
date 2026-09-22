#[derive(Debug, Clone, Copy)]
pub struct Article {
    pub title: String,
    pub description: String,
    pub content: Vec<String>,
    pub link: String,
    pub pub_date: String,
    pub categories: Vec<String>,
}
