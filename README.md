# News + Research Paper TUI (Ratatui + Rust)

A terminal-based news and research paper reader built with **Rust** and **ratatui**

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

* [x] Fetch research papers (e.g. arXiv or other APIs)
* [x] Parse and display paper metadata:
  * Title
  * Authors
  * Abstract
  * Publication date

---

### ⚙️ Architecture Improvements

* [x] Background fetching with async tasks (non-blocking UI)
* [x] Storing fetched data

---

### 🚀 Future Ideas

* [x] Offline mode with cached articles
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
