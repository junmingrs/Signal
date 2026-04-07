use std::collections::HashMap;

use tokio::sync::mpsc::Sender;
use tui_widget_list::ListState;

use crate::{
    database::sqlite::Db,
    services::arxiv::Arxiv,
    tui::display::Message,
    utils::{papers_model::PapersModel, sidebar::Sidebar},
};

pub struct Papers {
    pub items: Vec<PapersModel>,
    pub display_items: Vec<usize>, // list of item indices
    pub sidebar: Sidebar,
    pub scroll_offset: u16,
    pub max_scroll_offsets: HashMap<usize, u16>,
}

impl Papers {
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
            scroll_offset: 0,
            max_scroll_offsets: HashMap::<usize, u16>::new(),
        }
    }
    pub async fn fetch_titles_from_rss() -> Vec<PapersModel> {
        let xml_response = Arxiv::fetch_rss().await;
        Arxiv::parse(xml_response.clone())
    }
    pub fn fetch_papers_from_db(&mut self, tx: Sender<Message>, db: &Db) {
        let papers_model = db.fetch_papers();
        tokio::spawn(async move {
            tx.send(Message::PapersDBFetched(papers_model))
                .await
                .unwrap()
        });
    }
    pub fn fetch_papers_from_rss(&mut self, tx: Sender<Message>) {
        tokio::spawn(async move {
            tx.send(Message::PapersRSSFetched(
                Self::fetch_titles_from_rss().await,
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
    pub fn reload_sidebar(&mut self) {
        let mut items: Vec<String> = Vec::new();
        for i in self.display_items.iter() {
            items.push(self.items[*i].title.clone());
        }
        self.sidebar.titles = items;
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
