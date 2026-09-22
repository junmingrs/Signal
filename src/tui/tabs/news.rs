use std::{collections::HashMap, fmt};

use crate::{
    services::{
        businesstimes::{BT, NewsCategoryBT},
        cna::{CNA, NewsCategoryCNA},
        straitstimes::{NewsCategoryST, ST},
    },
    utils::{article::Article, sidebar::Sidebar},
};
use tui_widget_list::ListState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NewsSourceEnum {
    CNA(NewsCategoryCNA),
    StraitsTimes(NewsCategoryST),
    BusinessTimes(NewsCategoryBT),
}

impl fmt::Display for NewsSourceEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            NewsSourceEnum::CNA(_) => "CNA",
            NewsSourceEnum::StraitsTimes(_) => "StraitsTimes",
            NewsSourceEnum::BusinessTimes(_) => "BusinessTimes",
        };
        write!(f, "{}", s)
    }
}

#[derive(Copy)]
pub struct NewsSource {
    pub source_variant: NewsSourceEnum,
    category_index: usize,
}

impl NewsSource {
    pub fn new(source_variant: NewsSourceEnum) -> Self {
        Self {
            source_variant,
            category_index: 0,
        }
    }
}

pub struct News {
    cna: NewsSource,
    st: NewsSource,
    bt: NewsSource,
    current_source: NewsSourceEnum,
    pub sidebar: Sidebar,
    // NOTE: deal with scroll offsets later
    // pub scroll_offset: u16,
    // pub max_scroll_offsets: HashMap<usize, u16>,
}

impl News {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(None);
        let cna = NewsSource::new(NewsSourceEnum::CNA(NewsCategoryCNA::Latest(None)));
        let st = NewsSource::new(NewsSourceEnum::StraitsTimes(NewsCategoryST::Singapore(
            None,
        )));
        let bt = NewsSource::new(NewsSourceEnum::BusinessTimes(NewsCategoryBT::Singapore(
            None,
        )));
        Self {
            cna,
            st,
            bt,
            current_source: NewsSourceEnum::CNA(NewsCategoryCNA::Latest(None)),
            sidebar: Sidebar {
                display_titles: Vec::new(),
                state,
                focused: true,
            },
        }
    }
    pub async fn fetch_news(source_variant: NewsSourceEnum) -> Vec<Article> {
        match source_variant {
            NewsSourceEnum::CNA(cna) => {
                let xml_response = CNA::fetch_category(&cna).await;
                CNA::parse(xml_response.clone()).await
            }
            NewsSourceEnum::StraitsTimes(st) => {
                let xml_response = ST::fetch_category(&st).await;
                ST::parse(xml_response.clone(), st).await
            }
            NewsSourceEnum::BusinessTimes(bt) => {
                let xml_response = BT::fetch_category(&bt).await;
                BT::parse(xml_response.clone(), bt).await
            }
        }
    }
    pub fn get_current_news_source(&self) -> &NewsSource {
        match self.current_source {
            NewsSourceEnum::CNA(_) => &self.cna,
            NewsSourceEnum::StraitsTimes(_) => &self.st,
            NewsSourceEnum::BusinessTimes(_) => &self.bt,
        }
    }
    pub fn update_source_articles(&mut self, articles: Vec<Article>) {
        match self.current_source {
            NewsSourceEnum::CNA(cna) => cna.update_articles(articles),
            NewsSourceEnum::StraitsTimes(st) => &self.st,
            NewsSourceEnum::BusinessTimes(bt) => bt.,
        }
    }
    // NOTE: assume always selected
    pub fn get_current_article(&self) -> &Article {
        &self.items[self.display_items[self
            .sidebar
            .state
            .selected
            .expect("Assumed state is always selected")]]
        // match self.sidebar.state.selected {
        //     Some(i) => Some(&self.items[self.display_items[i]]),
        //     None => None,
        // }
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
        let mut titles: Vec<String> = Vec::new();
        for i in self.display_items.iter() {
            titles.push(self.items[*i].title.clone());
        }
        self.sidebar.display_titles = titles;
    }
    pub fn update_news_category(&mut self, next: bool) {
        if next {
            self.category.next();
        } else {
            self.category.previous();
        }
    }
    pub fn next(&mut self) {
        if self.display_items.is_empty() {
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
        if self.display_items.is_empty() {
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
        if let Some(i) = self.sidebar.state.selected
            && let Some(max) = self.max_scroll_offsets.get(&i)
            && self.scroll_offset < *max
        {
            self.scroll_offset += 1;
        }
    }
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }
}
