use std::collections::HashMap;

use crate::{
    services::cna::{CNA, NewsCategory},
    utils::cna_model::CNAModel,
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget, Wrap},
};
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
        // Block::from(self.text).style(self.style).render(area, buf);
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
}

impl Widget for &mut Sidebar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let builder = ListBuilder::new(|context| {
            let mut item = ListItem::new(self.titles[context.index].clone());
            if context.is_selected {
                item.style = Style::default().bg(Color::Gray);
            };
            let main_axis_size = 5; // what is this?
            (item, main_axis_size)
        });
        let item_count = self.titles.len();
        let list = ListView::new(builder, item_count).block(Block::default().borders(Borders::ALL));
        let state = &mut self.state;
        list.render(area, buf, state);
    }
}

pub struct News {
    pub items: Vec<CNAModel>,
    pub display_items: Vec<usize>, // list of item indices
    pub sidebar: Sidebar,
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
            },
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
        if let Some(curr_idx) = self.sidebar.state.selected {
            for (idx, val) in self.display_items.iter().enumerate() {
                if &curr_idx == val {
                    done = true;
                    self.sidebar.state.select(Some(idx));
                }
            }
        }
        if !done {
            self.sidebar.state.select(None);
        }
    }
    pub fn next(&mut self) {
        let i = match self.sidebar.state.selected() {
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
