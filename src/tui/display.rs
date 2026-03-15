use std::{fs, time::Duration};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::{
        self,
        event::{self, Event, KeyCode, KeyModifiers},
    },
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{
        Block, Borders, List, ListItem, Padding, Paragraph, Scrollbar, ScrollbarState, Wrap,
    },
};
use ratatui_textarea::TextArea;

use crate::{
    services::cna::NewsCategory,
    tui::app::{App, Focused, Mode, Tab},
    utils::fuzzy::fuzzy_match,
};

pub fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    // setup app
    let mut app = App::new();
    app.news_app.items = app
        .tokio_runtime
        .block_on(app.news_app.fetch_news(NewsCategory::Latest));
    let mut items_index = Vec::new();
    for i in 0..app.news_app.items.len() {
        items_index.push(i);
    }
    app.news_app.display_items = items_index;
    // setup search area
    let mut search_area = TextArea::default();
    search_area.set_block(Block::default().borders(Borders::ALL));
    loop {
        terminal.draw(|frame| {
            render(frame, &mut app, &search_area);
        })?;
        if crossterm::event::poll(Duration::from_millis(500))? {
            if let Event::Key(key) = event::read()? {
                if key.code.is_esc() {
                    app.mode = Mode::Normal
                }
                match app.mode {
                    Mode::Insert => {
                        // disables newline by blocking control and enter
                        match key.modifiers {
                            KeyModifiers::CONTROL => {
                                continue;
                            }
                            _ => {}
                        }
                        match key.code {
                            KeyCode::Enter => {
                                continue;
                            }
                            // prevent reloading when search area is empty
                            KeyCode::Backspace => {
                                if search_area.is_empty() {
                                    continue;
                                }
                            }
                            _ => {}
                        }
                        search_area.input(key);
                        let mut results = fuzzy_match(
                            search_area.lines().join("").to_string(),
                            app.news_app.items.clone(),
                        );
                        results.sort_by_key(|(s, _, _)| *s);
                        results.reverse();
                        app.news_app.display_items =
                            results.iter().map(|(_, _, i)| i.clone()).collect();
                        if search_area.is_empty() {
                            app.news_app.reset_display_items();
                        }
                        app.news_app.update_state();
                        // fs::write("output.txt", format!("{:#?}", results)).unwrap();
                    }
                    Mode::Normal => match key.code.as_char() {
                        Some('1') => app.tab = Tab::News,
                        Some('2') => app.tab = Tab::Papers,
                        Some('3') => app.tab = Tab::Custom,
                        Some('4') => fs::write(
                            "output.txt",
                            app.news_app.items[app.news_app.state.selected().clone().unwrap()]
                                .content
                                .clone()
                                .unwrap()
                                .iter()
                                .map(|x| x.to_string())
                                .collect::<String>(),
                        )
                        .unwrap(),
                        Some('i') => app.mode = Mode::Insert,
                        Some('v') => app.mode = Mode::Visual,
                        Some('h') => app.focused = Focused::Left,
                        Some('l') => app.focused = Focused::Right,
                        Some('j') => match app.focused {
                            Focused::Left => app.news_app.next(),
                            Focused::Right => app.news_app.scroll_down(),
                        },
                        Some('k') => match app.focused {
                            Focused::Left => app.news_app.previous(),
                            Focused::Right => app.news_app.scroll_up(),
                        },
                        Some('q') => break Ok(()),
                        _ => {}
                    },
                    Mode::Visual => {}
                }
            }
        }
    }
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

fn format_sidebar_item(title: &str, width: usize) -> String {
    // refactor: this can be done better
    // suggestion: split by words instead of characters
    title
        .chars()
        .collect::<Vec<char>>()
        .chunks(width)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<String>>()
        .join("\n")
}

fn testing_block(frame: &mut Frame, word: &str, selected: bool, layout: Rect) {
    frame.render_widget(
        Paragraph::new(word)
            .bg(if selected { Color::Green } else { Color::Reset })
            .block(Block::new().borders(Borders::ALL)),
        layout,
    );
}

fn render(frame: &mut Frame, app: &mut App, search_area: &TextArea) {
    let base_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(10), Constraint::Percentage(90)])
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
    let sidebar_layout =
        Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(bottom_layout[0]);
    testing_block(frame, "", false, base_layout[0]);
    testing_block(
        frame,
        match app.mode {
            Mode::Normal => "Normal",
            Mode::Insert => "Insert",
            Mode::Visual => "Visual",
        },
        false,
        base_layout[0],
    );
    for (idx, i) in [Tab::News, Tab::Papers, Tab::Custom].iter().enumerate() {
        testing_block(
            frame,
            match i {
                Tab::News => "News",
                Tab::Papers => "Papers",
                Tab::Custom => "Custom",
            },
            if &app.tab == i { true } else { false },
            tab_layout[idx].centered_vertically(Constraint::Length(3)),
        );
    }
    let list = List::new({
        let mut items = Vec::<ListItem>::new();
        for i in app.news_app.display_items.iter() {
            let text = format_sidebar_item(
                &app.news_app.items[*i].title,
                sidebar_layout[1].width as usize - 3,
            );
            items.push(ListItem::from(text));
            // TODO: add a block for each list item
            // and make it contain pub date and categories
            // this would be replaced by tui-widget-list
        }
        items
    })
    .highlight_style(Style::default().bg(Color::Yellow))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .padding(Padding::new(1, 0, 0, 0)),
    );
    frame.render_widget(search_area, sidebar_layout[0]);
    // fs::write("lol.txt", format!("{:#?}", list)).unwrap();
    frame.render_stateful_widget(list, sidebar_layout[1], &mut app.news_app.state);

    if let Some(i) = app.news_app.state.selected() {
        match &app.news_app.items[app.news_app.display_items[i]].content {
            Some(content) => {
                let viewport_height = bottom_layout[1].height;
                let inner_width = bottom_layout[1].width.saturating_sub(1);
                let total_lines: u16 = content
                    .iter()
                    .map(|c| count_wrapped_lines(c, inner_width))
                    .sum::<u16>()
                    + (content.len().saturating_sub(1) as u16);
                let max_scroll: u16;
                match app.news_app.max_scroll_offsets.get(&i) {
                    Some(scroll_offset) => {
                        max_scroll = *scroll_offset;
                    }
                    None => {
                        // Max scroll: how many lines we can scroll before the last line hits bottom
                        max_scroll = total_lines.saturating_sub(viewport_height);
                        app.news_app.max_scroll_offsets.insert(i, max_scroll);

                        // Clamp scroll_offset in case content changed
                        if app.news_app.scroll_offset > max_scroll {
                            app.news_app.scroll_offset = max_scroll;
                        }
                    }
                }
                let joined = content.join("\n\n");
                frame.render_widget(
                    Paragraph::new(joined)
                        .wrap(Wrap { trim: true })
                        .scroll((app.news_app.scroll_offset, 0))
                        .block(Block::new().borders(Borders::ALL)),
                    bottom_layout[1],
                );

                if total_lines > viewport_height {
                    let mut scrollbar_state = ScrollbarState::new(max_scroll as usize)
                        .position(app.news_app.scroll_offset as usize);
                    let scrollbar =
                        Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight);
                    frame.render_stateful_widget(scrollbar, bottom_layout[1], &mut scrollbar_state);
                }
            }
            None => {
                app.news_app.items[app.news_app.display_items[i]].content = Some(
                    app.tokio_runtime.block_on(
                        app.news_app
                            .fetch_content(&app.news_app.items[app.news_app.display_items[i]]),
                    ),
                );
            }
        }
    }
}
// Store display_items as vec of index in items.
