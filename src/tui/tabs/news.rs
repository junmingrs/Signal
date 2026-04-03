use std::{collections::HashMap, fmt};

use crate::{
    database::sqlite::Db,
    services::{
        businesstimes::{BT, NewsCategoryBT},
        cna::{CNA, NewsCategoryCNA},
        straitstimes::{NewsCategoryST, ST},
    },
    tui::display::Message,
    utils::{news_model::NewsModel, sidebar::Sidebar},
};
use tokio::sync::mpsc::Sender;
use tui_widget_list::ListState;

#[derive(Debug, Clone, PartialEq)]
pub enum NewsSource {
    CNA,
    StraitsTimes,
    BusinessTimes,
}

impl fmt::Display for NewsSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            NewsSource::CNA => "CNA",
            NewsSource::StraitsTimes => "StraitsTimes",
            NewsSource::BusinessTimes => "BusinessTimes",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NewsCategoryKind {
    CNA(NewsCategoryCNA),
    ST(NewsCategoryST),
    BT(NewsCategoryBT),
}

impl fmt::Display for NewsCategoryKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            NewsCategoryKind::CNA(_) => "CNA",
            NewsCategoryKind::ST(_) => "ST",
            NewsCategoryKind::BT(_) => "BT",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone)]
pub struct NewsCategory {
    pub source: NewsSource, // use the source to match NewsCategoryKind
    categories: Vec<NewsCategoryKind>,
    loaded_categories: Vec<bool>,
    index: usize, // use index to get category
}

impl NewsCategory {
    pub fn new(source: NewsSource) -> Self {
        let categories = Self::get_categories(&source);
        Self {
            source,
            loaded_categories: vec![false; categories.len()],
            categories: categories,
            index: 0,
        }
    }
    fn get_categories(source: &NewsSource) -> Vec<NewsCategoryKind> {
        let mut categories: Vec<NewsCategoryKind> = Vec::new();
        match source {
            NewsSource::CNA => {
                for category in NewsCategoryCNA::ALL.iter() {
                    categories.push(NewsCategoryKind::CNA(*category));
                }
            }
            NewsSource::StraitsTimes => {
                for category in NewsCategoryST::ALL.iter() {
                    categories.push(NewsCategoryKind::ST(*category));
                }
            }
            NewsSource::BusinessTimes => {
                for category in NewsCategoryBT::ALL.iter() {
                    categories.push(NewsCategoryKind::BT(*category));
                }
            }
        }
        categories
    }
    pub fn update_source(&mut self, source: NewsSource) {
        self.categories = Self::get_categories(&source);
        self.source = source;
        self.loaded_categories = vec![false; self.categories.len()];
        self.index = 0;
    }
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.categories.len();
    }
    pub fn previous(&mut self) {
        if self.index == 0 {
            self.index = self.categories.len() - 1;
        } else {
            self.index = (self.index - 1) % self.categories.len();
        }
    }
    pub fn get_current(&self) -> NewsCategoryKind {
        self.categories[self.index]
    }
    pub fn set_loaded(&mut self) {
        self.loaded_categories[self.index] = true;
    }
    pub fn is_loaded(&self) -> bool {
        self.loaded_categories[self.index]
    }
}

pub struct News {
    pub items: Vec<NewsModel>,
    pub display_items: Vec<usize>, // list of item indices
    pub sidebar: Sidebar,
    pub category: NewsCategory,
    pub scroll_offset: u16,
    pub max_scroll_offsets: HashMap<usize, u16>,
}

impl News {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(None);
        Self {
            items: Vec::new(),
            display_items: Vec::new(),
            sidebar: Sidebar {
                titles: Vec::new(),
                state,
                focused: true,
            },
            category: NewsCategory::new(NewsSource::CNA),
            scroll_offset: 0,
            max_scroll_offsets: HashMap::<usize, u16>::new(),
        }
    }
    pub async fn fetch_titles_from_rss(category: &NewsCategoryKind) -> Vec<NewsModel> {
        match category {
            NewsCategoryKind::CNA(cna) => {
                let xml_response = CNA::fetch_category(&cna).await;
                CNA::parse(xml_response.clone())
            }
            NewsCategoryKind::ST(st) => {
                let xml_response = ST::fetch_category(&st).await;
                ST::parse(xml_response.clone(), *st)
            }
            NewsCategoryKind::BT(bt) => {
                let xml_response = BT::fetch_category(&bt).await;
                BT::parse(xml_response.clone(), *bt)
            }
        }
    }
    pub async fn fetch_article_content(
        category: NewsCategory,
        news_model: &NewsModel,
        tx: Sender<Message>,
    ) {
        let content;
        match category.source {
            NewsSource::CNA => {
                let xml_response = CNA::fetch_page(&news_model.link).await;
                let document = CNA::webscrape(&xml_response);
                content = CNA::get_content(document);
            }
            NewsSource::StraitsTimes => {
                let xml_response = ST::fetch_page(&news_model.link).await;
                let document = ST::webscrape(&xml_response);
                content = ST::get_content(document);
            }
            NewsSource::BusinessTimes => {
                let xml_response = BT::fetch_page(&news_model.link).await;
                let document = BT::webscrape(&xml_response);
                content = BT::get_content(document);
            }
        }
        let model = news_model.clone();
        tokio::spawn(async move {
            tx.send(Message::NewsContentFetched(content, model))
                .await
                .expect("Could not fetch content");
        });
    }
    pub fn fetch_news_from_db(&mut self, tx: Sender<Message>, db: &Db) {
        let category = self.category.clone();
        let news_models = db.fetch_news_by_source_and_category(&category);
        tokio::spawn(async move {
            tx.send(Message::FetchedNewsArticles(news_models))
                .await
                .unwrap()
        });
    }
    pub fn fetch_latest_news_from_db(&mut self, tx: Sender<Message>, db: &Db) {
        let news_source = self.category.source.clone();
        let news_models = db.fetch_latest_news_by_source(news_source);
        tokio::spawn(async move {
            tx.send(Message::FetchedNewsArticles(news_models))
                .await
                .unwrap()
        });
    }
    pub fn fetch_news_from_rss(&mut self, tx: Sender<Message>) {
        let category = self.category.clone();
        let category_kind = category.get_current();
        tokio::spawn(async move {
            tx.send(Message::RSSFetched(
                Self::fetch_titles_from_rss(&category_kind).await,
            ))
            .await
            .unwrap();
        });
    }
    pub fn clear_items(&mut self) {
        self.items = Vec::new();
        self.sidebar.state.selected = None;
    }
    pub fn reset_display_items(&mut self) {
        let mut items_index = Vec::new();
        for i in 0..self.items.len() {
            items_index.push(i);
        }
        self.display_items = items_index; // maybe refactor to a oneliner
    }
    pub fn reload_sidebar(&mut self) {
        let mut items: Vec<String> = Vec::new();
        for i in self.display_items.iter() {
            items.push(self.items[*i].title.clone());
        }
        self.sidebar.titles = items;
    }
    pub fn update_news_category(&mut self, next: bool) {
        if next {
            self.category.next();
        } else {
            self.category.previous();
        }
    }
    pub fn next(&mut self) {
        if self.display_items.len() == 0 {
            return;
        }
        let i = match self.sidebar.state.selected {
            Some(i) => {
                if i >= self.display_items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.sidebar.state.select(Some(i));
        self.scroll_offset = 0;
    }
    pub fn previous(&mut self) {
        if self.display_items.len() == 0 {
            return;
        }
        let i = match self.sidebar.state.selected {
            Some(i) => {
                if i == 0 {
                    self.display_items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.sidebar.state.select(Some(i));
        self.scroll_offset = 0;
    }
    pub fn scroll_down(&mut self) {
        if let Some(i) = self.sidebar.state.selected {
            if let Some(max) = self.max_scroll_offsets.get(&i) {
                if self.scroll_offset < *max {
                    self.scroll_offset += 1;
                }
            }
        }
    }
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }
}
