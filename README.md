# News TUI (Ratatui + Rust)

A terminal-based news reader built with **Rust** and **ratatui**

---

## Current Status

### ✅ Implemented

* News section (core functionality)
* RSS/XML fetching and parsing
* Basic article list display
* Article content scraping (partial)
* Keyboard navigation (list selection)

---

## Roadmap

### 🟡 In Progress (News Section Improvements)

* [x] Switch between news categories (e.g. Latest, Sports, Business)
* [x] Display published date for each article
* [x] Display article category
* [x] Store news in database

---

### 🔵 Planned: Papers Section

* [ ] Fetch research papers (e.g. arXiv or other APIs)
* [ ] Parse and display paper metadata:

  * Title
  * Authors
  * Abstract
  * Publication date
* [ ] Add category filtering (e.g. CS, Math, Physics)

---

### 🟣 Planned: Custom Feeds

* [ ] Allow user-defined RSS feeds
* [ ] Add/remove feeds dynamically

---

### ⚙️ Architecture Improvements

* [x] Background fetching with async tasks (non-blocking UI)
* [ ] Better state management (separate UI vs data state)
* [ ] Storing fetched data

---

### 🚀 Future Ideas

* [ ] Offline mode with cached articles
* [ ] Notifications for new articles
* [ ] Bookmarking / saving articles
* [ ] Keyboard shortcut customization

---

## Tech Stack

* Rust
* ratatui (TUI framework)
* tokio (async runtime)
* reqwest (HTTP requests)

---
