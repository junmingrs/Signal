// render list
// render content

use std::collections::HashMap;

use crate::utils::cna_model::CNAModel;
use ratatui::widgets::ListState;

pub enum Focused {
    Left,
    Right,
}

pub struct News {
    pub items: Vec<CNAModel>,
    pub state: ListState,
    pub scroll_offset: u16,
    pub max_scroll_offsets: HashMap<usize, u16>,
}

impl News {
    pub fn new(items: Vec<CNAModel>) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            items,
            state,
            scroll_offset: 0,
            max_scroll_offsets: HashMap::<usize, u16>::new(),
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
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
                    self.items.len() - 1
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
                if self.scroll_offset < *max { // why cant it compare u16 with &u16
                    self.scroll_offset += 1;
                }
            }
            // if let Some(max) = self.max_scroll_offsets[i] {
            //     if self.scroll_offset < max {
            //         self.scroll_offset += 1
            //     }
            // }
        }
    }
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }
}
