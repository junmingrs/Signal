use std::{
    fs::{self},
    time::Duration,
};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::{
        self,
        event::{self, Event, KeyCode, KeyModifiers},
    },
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarState, Wrap},
};
use ratatui_textarea::TextArea;
use tokio::sync::mpsc::Sender;

use crate::{
    database::sqlite::Db,
    tui::{
        app::{App, Focused, Mode, Tab},
        tabs::news::{News, NewsCategoryKind, NewsSource},
    },
    utils::{fuzzy::fuzzy_match, helper::is_latest, news_model::NewsModel},
};

pub enum Message {
    // save to db
    RSSFetched(Vec<NewsModel>),
    RequireNewsContent(NewsModel),
    NewsContentFetched(Vec<String>, NewsModel),
    // fetch from db
    FetchedNewsArticles(Vec<NewsModel>),
    RequireNewsArticles(bool), // latest or not
}

pub fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    // setup db
    let db = Db::new();
    // setup app
    let mut app = App::new();
    app.news_app.fetch_news_from_rss(app.tx.clone());
    // setup search area
    let mut search_area = TextArea::default();
    loop {
        // handle async messages
        while let Ok(msg) = app.rx.try_recv() {
            match msg {
                Message::RSSFetched(items) => {
                    app.news_app.reset_display_items();
                    app.news_app.reload_sidebar();
                    for item in items {
                        let tx_background_fetch = app.tx.clone();
                        tokio::spawn(async move {
                            tx_background_fetch
                                .send(Message::RequireNewsContent(item))
                                .await
                                .expect("tx_background_fetch failed to send message");
                        });
                    }
                }
                Message::NewsContentFetched(content, mut news_model) => {
                    news_model.content = Some(content);
                    db.save_news(news_model);
                }
                Message::RequireNewsContent(news_model) => {
                    let tx_fetch_content = app.tx.clone();
                    let category = app.news_app.category.clone();
                    tokio::spawn(async move {
                        News::fetch_article_content(category, &news_model, tx_fetch_content).await;
                    });
                }
                Message::FetchedNewsArticles(news_models) => {
                    if news_models.len() == 0 {
                        app.news_app.fetch_news_from_rss(app.tx.clone());
                    } else {
                        app.news_app.items = news_models;
                        app.news_app.reset_display_items();
                        app.news_app.reload_sidebar();
                        app.news_app.sidebar.state.selected = Some(0);
                    }
                }
                Message::RequireNewsArticles(latest) => {
                    if latest {
                        app.news_app.fetch_latest_news_from_db(app.tx.clone(), &db);
                    } else {
                        app.news_app.fetch_news_from_db(app.tx.clone(), &db);
                    }
                }
            }
        }
        // handle input
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
                        app.news_app.reload_sidebar();
                        app.news_app.sidebar.state.selected = Some(0);
                    }
                    Mode::Normal => {
                        // use tab to cycle categories
                        let tx_normal = app.tx.clone();
                        match key.code {
                            KeyCode::Tab => match app.tab {
                                Tab::News => {
                                    app.news_app.update_news_category(true);
                                    app.news_app.clear_items();
                                    if !app.news_app.category.is_loaded() {
                                        app.news_app.fetch_news_from_rss(tx_normal);
                                    } else if is_latest(app.news_app.category.get_current()) {
                                        app.news_app.fetch_latest_news_from_db(tx_normal, &db);
                                    } else {
                                        app.news_app.fetch_news_from_db(tx_normal, &db);
                                    }
                                    app.news_app.category.set_loaded();
                                }
                                Tab::Papers => {}
                                Tab::Custom => {}
                            },
                            KeyCode::BackTab => match app.tab {
                                Tab::News => {
                                    app.news_app.update_news_category(false);
                                    if is_latest(app.news_app.category.get_current()) {
                                        app.news_app.fetch_latest_news_from_db(tx_normal, &db);
                                    } else {
                                        app.news_app.fetch_news_from_db(tx_normal, &db);
                                    }
                                }
                                Tab::Papers => {}
                                Tab::Custom => {}
                            },
                            KeyCode::Char('1') => app.tab = Tab::News,
                            KeyCode::Char('2') => app.tab = Tab::Papers,
                            KeyCode::Char('3') => app.tab = Tab::Custom,
                            KeyCode::Char('4') => fs::write(
                                "output.txt",
                                app.news_app.items[app.news_app.sidebar.state.selected.unwrap()]
                                    .content
                                    .clone()
                                    .unwrap()
                                    .iter()
                                    .map(|x| x.to_string())
                                    .collect::<String>(),
                            )
                            .unwrap(),
                            KeyCode::Char('5') => db.save_news_batch(app.news_app.items.clone()),
                            KeyCode::Char('6') => app.news_app.fetch_news_from_rss(tx_normal), // reload
                            KeyCode::Char('c') => {
                                app.news_app.category.update_source(NewsSource::CNA);
                                app.news_app.clear_items();
                                // app.news_app.fetch_news_from_rss(tx_normal);
                            }
                            KeyCode::Char('s') => {
                                app.news_app
                                    .category
                                    .update_source(NewsSource::StraitsTimes);
                                app.news_app.clear_items();
                                // app.news_app.fetch_news_from_rss(tx_normal);
                            }
                            KeyCode::Char('b') => {
                                app.news_app
                                    .category
                                    .update_source(NewsSource::BusinessTimes);
                                app.news_app.clear_items();
                                // app.news_app.fetch_news_from_rss(tx_normal);
                            }
                            KeyCode::Char('i') => app.mode = Mode::Insert,
                            KeyCode::Char('v') => app.mode = Mode::Visual,
                            KeyCode::Char('h') => {
                                app.focused = Focused::Left;
                                app.news_app.sidebar.focused = true;
                            }
                            KeyCode::Char('l') => {
                                app.focused = Focused::Right;
                                app.news_app.sidebar.focused = false;
                            }
                            KeyCode::Char('j') => match app.focused {
                                Focused::Left => app.news_app.next(),
                                Focused::Right => app.news_app.scroll_down(),
                            },
                            KeyCode::Char('k') => match app.focused {
                                Focused::Left => app.news_app.previous(),
                                Focused::Right => app.news_app.scroll_up(),
                            },
                            KeyCode::Char('q') => break Ok(()),
                            _ => {}
                        }
                    }
                    Mode::Visual => {}
                }
            }
        }
        terminal.draw(|frame| {
            render(frame, &mut app, &mut search_area);
        })?;
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

fn bordered_block(frame: &mut Frame, word: &str, selected: bool, layout: Rect) {
    frame.render_widget(
        Paragraph::new(word)
            .fg(if selected {
                Color::Yellow
            } else {
                Color::Reset
            })
            .block(Block::new().borders(Borders::ALL)),
        layout,
    );
}

fn render(frame: &mut Frame, app: &mut App, search_area: &mut TextArea) {
    let base_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(10), Constraint::Percentage(90)])
        .split(frame.area());
    let body_layout = Layout::default()
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
        Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(body_layout[0]);
    bordered_block(frame, "", false, base_layout[0]);
    bordered_block(
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
        bordered_block(
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
    search_area.set_block(Block::default().borders(Borders::ALL).border_style(
        Style::default().fg(if let Mode::Insert = app.mode {
            Color::Yellow
        } else {
            Color::Reset
        }),
    ));
    frame.render_widget(&*search_area, sidebar_layout[0]);
    match app.tab {
        Tab::News => {
            render_news(
                frame,
                &mut app.news_app,
                body_layout[1],
                sidebar_layout[1],
                app.tx.clone(),
            );
        }
        Tab::Papers => {}
        Tab::Custom => {}
    }
}

fn render_news(
    frame: &mut Frame,
    news_app: &mut News,
    bottom_layout: Rect,
    sidebar_list: Rect,
    tx: Sender<Message>,
) {
    let sidebar_category_list =
        Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(sidebar_list);
    frame.render_widget(&mut news_app.sidebar, sidebar_category_list[1]);

    let bottom =
        Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(bottom_layout);
    let bottom_top = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .flex(Flex::SpaceBetween)
        .split(bottom[0]);

    let category_and_source =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(sidebar_category_list[0]);

    frame.render_widget(
        Paragraph::new(match news_app.category.get_current() {
            NewsCategoryKind::CNA(cna) => cna.to_string(),
            NewsCategoryKind::ST(st) => st.to_string(),
            NewsCategoryKind::BT(bt) => bt.to_string(),
        })
        .block(Block::new().borders(Borders::ALL).title("Category")),
        category_and_source[0],
    );

    frame.render_widget(
        Paragraph::new(match news_app.category.source {
            NewsSource::CNA => "CNA",
            NewsSource::StraitsTimes => "StraitsTimes",
            NewsSource::BusinessTimes => "BusinessTimes",
        })
        .block(Block::new().borders(Borders::ALL).title("Source")),
        category_and_source[1],
    );

    match news_app.sidebar.state.selected {
        Some(i) => {
            if news_app.display_items.len() == 0 || news_app.items.len() == 0 {
                return;
            }
            match &news_app.items[news_app.display_items[i]].content {
                Some(content) => {
                    bordered_block(
                        frame,
                        &news_app.items[news_app.display_items[i]].pub_date,
                        false,
                        bottom_top[0],
                    );
                    bordered_block(
                        frame,
                        &news_app.items[news_app.display_items[i]]
                            .categories
                            .join(", "),
                        false,
                        bottom_top[1],
                    );

                    let viewport_height = bottom[1].height;
                    let inner_width = bottom[1].width.saturating_sub(1);
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
                            .scroll((news_app.scroll_offset, 0))
                            .block(Block::new().borders(Borders::ALL).border_style(
                                Style::default().fg(if !news_app.sidebar.focused {
                                    Color::Yellow
                                } else {
                                    Color::Reset
                                }),
                            )),
                        bottom[1],
                    );

                    if total_lines > viewport_height {
                        let mut scrollbar_state = ScrollbarState::new(max_scroll as usize)
                            .position(news_app.scroll_offset as usize);
                        let scrollbar =
                            Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight);
                        frame.render_stateful_widget(scrollbar, bottom[1], &mut scrollbar_state);
                    }
                }
                None => {
                    if let Some(i) = news_app.sidebar.state.selected {
                        let news_model = news_app.items[news_app.display_items[i]].clone();
                        tokio::spawn(async move {
                            tx.send(Message::RequireNewsContent(news_model))
                                .await
                                .unwrap();
                        });
                    }
                }
            }
        }
        None => {
            let latest = is_latest(news_app.category.get_current());
            tokio::spawn(
                async move { tx.send(Message::RequireNewsArticles(latest)).await.unwrap() },
            );
        }
    }
}
