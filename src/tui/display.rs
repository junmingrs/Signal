use std::{fs, time::Duration};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::{
        self,
        event::{self, Event},
    },
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarState, Wrap},
};
use tokio::runtime::Runtime;

use crate::{
    services::cna::{CNA, NewsCategory},
    tui::tabs::news::{self, Focused, News},
    utils::cna_model::CNAModel,
};

#[derive(PartialEq)]
pub enum Tab {
    News,
    Papers,
    Custom,
}

pub fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut tab = Tab::News;
    let rt = tokio::runtime::Runtime::new().unwrap();
    let items = rt.block_on(fetch_news());
    let mut news_app = news::News::new(items);
    let mut focused: Focused = Focused::Left;
    loop {
        terminal.draw(|frame| {
            render(frame, &tab, &mut news_app, &rt);
        })?;
        if crossterm::event::poll(Duration::from_millis(500))? {
            if let Event::Key(key) = event::read()? {
                match key.code.as_char() {
                    Some('1') => tab = Tab::News,
                    Some('2') => tab = Tab::Papers,
                    Some('3') => tab = Tab::Custom,
                    Some('4') => fs::write(
                        "output.txt",
                        news_app.items[news_app.state.selected().clone().unwrap()]
                            .content
                            .clone()
                            .unwrap()
                            .iter()
                            .map(|x| x.to_string())
                            .collect::<String>(),
                    )
                    .unwrap(),
                    Some('h') => focused = Focused::Left,
                    Some('l') => focused = Focused::Right,
                    Some('j') => match focused {
                        Focused::Left => news_app.next(),
                        Focused::Right => news_app.scroll_down(),
                    },
                    Some('k') => match focused {
                        Focused::Left => news_app.previous(),
                        Focused::Right => news_app.scroll_up(),
                    },
                    Some('q') => break Ok(()),
                    _ => {}
                }
            }
        }
    }
}

pub async fn fetch_news() -> Vec<CNAModel> {
    let xml_response = CNA::fetch_category(NewsCategory::Latest).await;
    CNA::parse(xml_response.clone())
}

pub async fn fetch_content(cna_model: &CNAModel) -> Vec<String> {
    let xml_response = CNA::fetch_page(&cna_model.link).await;
    let document = CNA::webscrape(&xml_response);
    CNA::get_content(document)
}

fn count_wrapped_lines(text: &str, width: u16) -> u16 {
    if width == 0 {
        return 1;
    }
    let width = width as usize;
    let mut lines = 1u16;
    let mut current_len = 0usize;
    for word in text.split_whitespace() {
        let word_len = word.chars().count();
        if current_len == 0 {
            current_len = word_len;
        } else if current_len + 1 + word_len > width {
            lines += 1;
            current_len = word_len;
        } else {
            current_len += 1 + word_len;
        }
    }
    lines.max(1)
}

fn testing_block(frame: &mut Frame, word: &str, selected: bool, layout: Rect) {
    frame.render_widget(
        Paragraph::new(word)
            .bg(if selected { Color::Green } else { Color::Reset })
            .block(Block::new().borders(Borders::ALL)),
        layout,
    );
}

fn render(frame: &mut Frame, tab: &Tab, news_app: &mut News, rt: &Runtime) {
    let base_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(frame.area());
    let bottom_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(base_layout[1]);
    let tab_layout = Layout::horizontal([
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(10),
    ])
    .flex(Flex::Center)
    .spacing(2)
    .split(base_layout[0]);
    testing_block(frame, "", false, base_layout[0]);
    for (idx, i) in [Tab::News, Tab::Papers, Tab::Custom].iter().enumerate() {
        testing_block(
            frame,
            match i {
                Tab::News => "News",
                Tab::Papers => "Papers",
                Tab::Custom => "Custom",
            },
            if tab == i { true } else { false },
            tab_layout[idx].centered_vertically(Constraint::Length(3)),
        );
    }
    let list = List::new(
        news_app
            .items
            .iter()
            .map(|x| ListItem::from(x.title.clone())),
    )
    .highlight_style(Style::default().bg(Color::Yellow))
    .block(Block::default().borders(Borders::ALL));
    frame.render_stateful_widget(list, bottom_layout[0], &mut news_app.state);

    if let Some(i) = news_app.state.selected() {
        match &news_app.items[i].content {
            Some(content) => {
                let viewport_height = bottom_layout[1].height;
                let inner_width = bottom_layout[1].width.saturating_sub(1);
                let total_lines: u16 = content
                    .iter()
                    .map(|c| count_wrapped_lines(c, inner_width))
                    .sum::<u16>()
                    + (content.len().saturating_sub(1) as u16);
                let max_scroll: u16;
                match news_app.max_scroll_offsets.get(&i) {
                    Some(scroll_offset) => {
                        max_scroll = *scroll_offset;
                    }
                    None => {
                        // Max scroll: how many lines we can scroll before the last line hits bottom
                        max_scroll = total_lines.saturating_sub(viewport_height);
                        news_app.max_scroll_offsets.insert(i, max_scroll);

                        // Clamp scroll_offset in case content changed
                        if news_app.scroll_offset > max_scroll {
                            news_app.scroll_offset = max_scroll;
                        }
                    }
                }
                let joined = content.join("\n\n");
                frame.render_widget(
                    Paragraph::new(joined)
                        .wrap(Wrap { trim: true })
                        .scroll((news_app.scroll_offset, 0)),
                    bottom_layout[1],
                );

                if total_lines > viewport_height {
                    let mut scrollbar_state = ScrollbarState::new(max_scroll as usize)
                        .position(news_app.scroll_offset as usize);
                    let scrollbar =
                        Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight);
                    frame.render_stateful_widget(scrollbar, bottom_layout[1], &mut scrollbar_state);
                }
            }
            None => {
                news_app.items[i].content = Some(rt.block_on(fetch_content(&news_app.items[i])));
            }
        }
    }
}
