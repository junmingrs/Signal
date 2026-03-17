use tokio::runtime::Runtime;

use crate::tui::tabs::news::News;

pub enum Focused {
    Left,
    Right,
}

pub enum Mode {
    Normal,
    Insert, // should only work for the search feature
    Visual, // should only work in content to select content
}

#[derive(PartialEq)]
pub enum Tab {
    News,
    Papers,
    Custom,
}

pub struct App {
    pub focused: Focused,
    pub mode: Mode,
    pub tab: Tab,
    pub news_app: News,
    // pub papers_app: Papers,
    // pub custom_app: Custom,
    pub tokio_runtime: Runtime,
}

impl App {
    pub fn new() -> Self {
        let rt = tokio::runtime::Runtime::new().unwrap();
        // let items = rt.block_on(News::fetch_news(NewsCategory::Latest));
        Self {
            focused: Focused::Left,
            mode: Mode::Normal,
            tab: Tab::News,
            news_app: News::new(),
            tokio_runtime: rt,
        }
    }
}
