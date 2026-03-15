use std::collections::HashMap;

use crate::{
    services::cna::{CNA, NewsCategory},
    utils::cna_model::CNAModel,
};
use ratatui::widgets::ListState;

pub struct News {
    pub items: Vec<CNAModel>,
    pub display_items: Vec<usize>, // list of item indices
    pub state: ListState,
    pub scroll_offset: u16,
    pub max_scroll_offsets: HashMap<usize, u16>,
}

impl News {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        // let mut items_index = Vec::new();
        // for i in 0..items.len() {
        //     items_index.push(i);
        // }
        Self {
            items: Vec::new(),
            display_items: Vec::new(),
            state,
            scroll_offset: 0,
            max_scroll_offsets: HashMap::<usize, u16>::new(),
        }
    }
    pub async fn fetch_news(&self, category: NewsCategory) -> Vec<CNAModel> {
        let xml_response = CNA::fetch_category(category).await;
        CNA::parse(xml_response.clone())
    }

    pub async fn fetch_content(&self, cna_model: &CNAModel) -> Vec<String> {
        let xml_response = CNA::fetch_page(&cna_model.link).await;
        let document = CNA::webscrape(&xml_response);
        CNA::get_content(document)
    }
    pub fn reset_display_items(&mut self) {
        let mut items_index = Vec::new();
        for i in 0..self.items.len() {
            items_index.push(i);
        }
        self.display_items = items_index; // maybe refactor to a oneliner
    }
    pub fn update_state(&mut self) {
        let mut done = false;
        if let Some(curr_idx) = self.state.selected() {
            for (idx, val) in self.display_items.iter().enumerate() {
                if &curr_idx == val {
                    done = true;
                    self.state.select(Some(idx));
                }
            }
        }
        if !done {
            self.state.select(None);
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.display_items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_offset = 0;
    }
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.display_items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_offset = 0;
    }
    pub fn scroll_down(&mut self) {
        if let Some(i) = self.state.selected() {
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
