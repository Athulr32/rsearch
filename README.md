# TF-IDF Search Engine in Rust

A fast, file-based search engine built in **Rust** that uses the **TF-IDF** algorithm to rank results across local documents. It supports parsing and indexing content from **PDF** and **HTML** files, and stores tokenized word frequencies in **RocksDB**.

> ⚠️ Note: This project is a 3-year-old prototype written for experimentation and learning. The code may be unoptimized or outdated.

---

## 🔍 Features

- ✅ PDF and HTML file parsing  
- ✅ Custom character-level lexer and tokenizer  
- ✅ TF-IDF based scoring  
- ✅ RocksDB-backed document store  

---

## 📦 Dependencies

- `lopdf` – for parsing PDFs  
- `scraper` – for parsing HTML  
- `rocksdb` – to store document term frequencies  
- `unicode-normalization` – for text normalization

Install dependencies via:

```bash
cargo build
```

---

## 🚀 Usage

### 1. Place your `.pdf` and `.html` files

Put your documents in a directory (you can change the `dir_path` in `main.rs`):

```rust
let dir_path = Path::new("/Users/athul/");
```

### 2. Run the search engine

```bash
cargo run
```

It will:
- Crawl and index all supported files recursively
- Store word frequency data in RocksDB
- Ask for a search term
- Rank and print relevant documents based on TF-IDF score

---


## 📜 License

MIT — see [LICENSE](LICENSE)