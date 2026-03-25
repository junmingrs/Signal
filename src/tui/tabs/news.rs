use std::{collections::HashMap, fmt};

use crate::{
    database::sqlite::Db,
    services::{
        cna::{CNA, NewsCategoryCNA},
        straitstimes::NewsCategorySR,
    },
    tui::display::Message,
    utils::news_model::NewsModel,
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget, Wrap},
};
use tokio::sync::mpsc::Sender;
use tui_widget_list::{ListBuilder, ListState, ListView};

struct ListItem {
    pub text: String,
    pub style: Style,
}

impl ListItem {
    pub fn new<T: Into<String>>(text: T) -> Self {
        Self {
            text: text.into(),
            style: Style::default(),
        }
    }
}

impl Widget for ListItem {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.text)
            .style(self.style)
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL))
            .render(area, buf);
    }
}

pub struct Sidebar {
    pub titles: Vec<String>,
    pub state: ListState,
    pub focused: bool,
}

impl Widget for &mut Sidebar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let builder = ListBuilder::new(|context| {
            let mut item = ListItem::new(self.titles[context.index].clone());
            if context.is_selected {
                item.style = Style::default().fg(Color::Yellow);
            };
            let main_axis_size = 5; // what is this?
            (item, main_axis_size)
        });
        let item_count = self.titles.len();
        let list = ListView::new(builder, item_count).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if self.focused {
                    Color::Yellow
                } else {
                    Color::Reset
                })),
        );
        let state = &mut self.state;
        list.render(area, buf, state);
    }
}

pub enum NewsSource {
    CNA,
    StraitsTimes,
    BusinessTimes,
    WallStreetJournal,
}

#[derive(Clone, Copy)]
pub enum NewsCategoryKind {
    CNA(NewsCategoryCNA),
    SR(NewsCategorySR),
}

impl fmt::Display for NewsCategoryKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            NewsCategoryKind::CNA(_) => "CNA",
            NewsCategoryKind::SR(_) => "SR",
        };
        write!(f, "{}", s)
    }
}

pub struct NewsCategory {
    source: NewsSource, // will be enum of specific news rss feeds in the future
    categories: Vec<NewsCategoryKind>, // || Vec<NewsCategorySR>
    index: usize,
}

impl NewsCategory {
    pub fn new(source: NewsSource) -> Self {
        let mut categories: Vec<NewsCategoryKind> = Vec::new();
        match source {
            NewsSource::CNA => {
                for category in NewsCategoryCNA::ALL.iter() {
                    categories.push(NewsCategoryKind::CNA(*category));
                }
            }
            NewsSource::StraitsTimes => {}
            NewsSource::BusinessTimes => {}
            NewsSource::WallStreetJournal => {}
        }
        Self {
            source,
            categories: categories,
            index: 0,
        }
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
        state.select(Some(0));
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
    pub async fn fetch_articles_from_rss(category: &NewsCategoryKind) -> Vec<NewsModel> {
        match category {
            NewsCategoryKind::CNA(cna) => {
                let xml_response = CNA::fetch_category(&cna).await;
                CNA::parse(xml_response.clone())
            }
            NewsCategoryKind::SR(sr) => Vec::new(),
        }
    }
    pub async fn fetch_article_content(news_model: &NewsModel, tx: Sender<Message>, idx: usize) {
        let xml_response = CNA::fetch_page(&news_model.link).await;
        let document = CNA::webscrape(&xml_response);
        let content = CNA::get_content(document);
        tokio::spawn(async move {
            tx.send(Message::NewsContentFetched((content, idx)))
                .await
                .expect("Could not fetch content");
        });
    }
    pub fn fetch_articles_by_category(&mut self, tx: Sender<Message>, db: &Db) {
        let category_kind = self.category.get_current().clone();
        self.items = db.fetch_news_by_category(&category_kind);
        tokio::spawn(async move {
            tx.send(Message::NewsFetched(
                Self::fetch_articles_from_rss(&category_kind).await,
            ))
            .await
            .unwrap();
        });
    }
    pub fn fetch_articles(&mut self, tx: Sender<Message>, db: &Db) {
        self.items = db.fetch_news(10);
        let category_kind = self.category.get_current();
        tokio::spawn(async move {
            tx.send(Message::NewsFetched(
                Self::fetch_articles_from_rss(&category_kind).await,
            ))
            .await
            .unwrap();
        });
    }
    pub fn reset_display_items(&mut self) {
        let mut items_index = Vec::new();
        for i in 0..self.items.len() {
            items_index.push(i);
        }
        self.display_items = items_index; // maybe refactor to a oneliner
    }
    // pub fn update_state(&mut self) {
    //     let mut done = false;
    //     if let Some(curr_idx) = self.sidebar.state.selected {
    //         for (idx, val) in self.display_items.iter().enumerate() {
    //             if &curr_idx == val {
    //                 done = true;
    //                 self.sidebar.state.select(Some(idx));
    //             }
    //         }
    //     }
    //     if !done {
    //         self.sidebar.state.select(None);
    //     }
    // }
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
    pub fn reload_sidebar(&mut self) {
        let mut items: Vec<String> = Vec::new();  
        for i in self.display_items.iter() {
            items.push(self.items[*i].title.clone());
        }
        self.sidebar.titles = items;
    }
    pub fn update_news_category(&mut self, next: bool, tx: Sender<Message>, db: &Db) {
        if next {
            self.category.next();
        } else {
            self.category.previous();
        }
        self.fetch_articles(tx, &db);
        self.reload_sidebar();
    }
}
