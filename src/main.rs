use lopdf::{Document, Object};
use rocksdb::{IteratorMode, Options, DB};
use scraper::{Html, Selector};
use std::fs::{self, DirEntry, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::{collections::HashMap, ffi::OsStr, path::Path};
use unicode_normalization::UnicodeNormalization;

//This should iterate through the string then tokenise each part
// And bark the tokens out
struct Lexer<'a> {
    content: &'a [char], // Will store the pointer to the current starting letter of the string to the end
}

enum FileExtension {
    PDF,
    HTML,
    XML,
}

impl<'a> Lexer<'a> {
    fn new(content: &'a [char]) -> Self {
        Self { content }
    }

    fn trim_whitespace(&mut self) {
        //Discard the tokens if it is a whitespace
        while self.content.len() > 0 && self.content[0].is_whitespace() {
            self.content = &self.content[1..];
        }
    }

    fn trim_non_alphanumeric(&mut self) {
        //Discard the tokens if it is a whitespace
        while self.content.len() > 0 && !self.content[0].is_alphanumeric() {
            self.content = &self.content[1..];
        }
    }

    // Lexer should be called in a loop and this function will be called each time
    fn tokenise(&mut self) -> Option<&'a [char]> {
        if self.content.len() == 0 {
            return None;
        }

        // Check if the char is numeric
        if self.content[0].is_numeric() {
            //Loop till a non numeric is found
            let mut i = 0;
            while self.content.len() > i && self.content[i].is_numeric() {
                i = i + 1;
            }
            let token = &self.content[0..i];
            self.content = &self.content[i..];

            return Some(token);
        }

        //Check if the char is an alphabet
        if self.content[0].is_alphabetic() {
            //Loop till a non numeric is found
            let mut i = 0;
            while self.content.len() > i && self.content[i].is_alphabetic() {
                i = i + 1;
            }
            let token = &self.content[0..i];
            self.content = &self.content[i..];

            return Some(token);
        }

        // Else Discard the token
        // We make sure all the non alphanumerics are trimmed out
        return None;
    }
}

//Implement a iterator for Lexer to tokenise in a loop
impl<'a> Iterator for Lexer<'a> {
    //The return Item from the iterator
    type Item = &'a [char];

    fn next(&mut self) -> Option<Self::Item> {
        self.trim_whitespace();
        self.trim_non_alphanumeric();
        let current = self.tokenise();
        return current;
    }
}

type WordsFreq = HashMap<String, usize>;
type DocWordsFreq = HashMap<String, WordsFreq>;

fn main() {
    let search = "Convocation";
    let dir_path = Path::new("/Users/athul/");

    let mut file_words_map: DocWordsFreq = HashMap::new();

    //Visit all Directories and Index the files
    let _ = visit_dirs(dir_path, &tokenize_file_content).map_err(|e| {
        println!("{e}");
    });

    // Read Data from RocksDb
    let path = "./rocksdb";
    let db = DB::open_default(path).unwrap();

    // Create an iterator
    let iter = db.iterator(IteratorMode::Start);

    for item in iter {
        match item {
            Ok((key, value)) => {
                let path = String::from_utf8_lossy(&key);
                let value = serde_json::from_slice::<WordsFreq>(&value).unwrap();
                println!("{:?}", value);
                file_words_map.insert(path.to_string(), value);
            }
            Err(_) => {}
        }
    }
    let search_arr = search.chars().collect::<Vec<char>>();

    let mut rank: HashMap<String, f64> = HashMap::new();

    //Loop through all the indexed files
    // Calculate the TF in each doc for the user entered query
    for (doc, term_freq) in &file_words_map {
        let mut total_tf: f64 = 0.0;
        let lexer = Lexer::new(&search_arr);
        for token in lexer {
            let foo: String = token.into_iter().collect();
            let tf: f64 = calculate_tf(&foo, term_freq) * calculate_idf(&foo, &file_words_map);
            total_tf += tf;
        }
        rank.insert(doc.to_string(), total_tf);
    }

    // Convert the HashMap to a Vec of tuples
    let mut sorted: Vec<_> = rank.into_iter().collect();

    // Sort the Vec by value (second element of the tuple)

    // Print the sorted result
    for (key, value) in sorted {
        println!("{}: {}", key, value);
    }

    //Search
}

fn calculate_tf(term: &str, term_freq: &WordsFreq) -> f64 {
    let term_frequency_in_doc = (term_freq.get(term).unwrap_or(&0).clone()) as f64;
    let total_term_frequency_in_doc: usize = term_freq.values().sum();
    let tf = term_frequency_in_doc as f64 / (total_term_frequency_in_doc) as f64;
    return tf;
}

fn calculate_idf(term: &str, term_feq_index: &DocWordsFreq) -> f64 {
    let total_docs = (term_feq_index.len() + 1) as f64;
    let total_docs_term_appears = (term_feq_index
        .values()
        .filter(|t| t.contains_key(term))
        .count()
        + 1) as f64;
    (total_docs / total_docs_term_appears).log10()
}

fn get_file_extension(filename: &str) -> Option<FileExtension> {
    //Check the signature
    let file = File::open(filename);
    if file.is_err() {
        return None;
    }

    let mut file_open = file.unwrap();
    let mut buffer = [0u8; 4];
    let read = file_open
        .read_exact(&mut buffer)
        .map_err(|e| println!("Failed to Read Bytes"));

    if read.is_err() {
        return None;
    }
    file_open.seek(SeekFrom::Start(0)).unwrap(); // Reset file pointer

    match &buffer {
        b"%PDF" => Some(FileExtension::PDF),
        b"<xml" | b"<?xm" => Some(FileExtension::XML),
        b"<htm" => Some(FileExtension::HTML),
        // Add more signatures here
        _ => None,
    }
}

fn visit_dirs(dir: &Path, cb: &dyn Fn(&DirEntry)) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

fn tokenize_file_content(dir: &DirEntry) {
    let file_path = dir.path();

    let file_extension = get_file_extension(file_path.to_str().unwrap());
    if file_extension.is_none() {
        return;
    }
    let file_extension = file_extension.unwrap();
    let mut content = String::new();

    match file_extension {
        FileExtension::HTML => {
            println!("Found HTML");
            content = parse_html(file_path.clone());
        }
        FileExtension::PDF => {
            println!("Found PDF {:?}", file_path.to_str());
            content = parse_pdf(file_path.clone());
        }
        FileExtension::XML => {
            println!("Found Something Else");
            return;
        }
    }

    if content.len() == 0 {
        return;
    }
    let char_arr = content.chars().collect::<Vec<char>>();
    let lexer = Lexer::new(&char_arr);

    let mut words_count: WordsFreq = HashMap::new();

    for token in lexer {
        let foo: String = token.into_iter().collect();
        let count = words_count.get(&foo);
        if let Some(c) = count {
            let total_count = c + 1;
            words_count.insert(foo, total_count);
        } else {
            words_count.insert(foo, 1);
        }
    }

    if words_count.len() == 0 {
        return;
    }

    //Store In DB
    let path = "./rocksdb";
    let db = DB::open_default(path).unwrap();
    let serialise_term_freq = serde_json::to_string(&words_count).unwrap();
    db.put(file_path.to_str().unwrap(), serialise_term_freq)
        .unwrap();
}

fn parse_html(file_path: PathBuf) -> String {
    let file_string = fs::read_to_string(file_path).unwrap();
    let mut content = String::new();
    let document = Html::parse_document(&file_string);

    // Create a selector for all text nodes
    let selector = Selector::parse("body").expect("Failed to create selector");

    // Extract and print all text content
    for element in document.select(&selector) {
        let text = element.text().collect::<Vec<_>>().join(" ");
        content = content + &text
    }

    return content;
}

fn parse_pdf(file_path: PathBuf) -> String {
    let content = String::new();
    // Load the PDF document
    let doc = Document::load(file_path).unwrap();

    // Extract text from all pages
    let mut text = String::new();
    for (page_number, page_id) in doc.get_pages().iter() {
        let page = doc.get_page_content(*page_id).unwrap();
        let resources = doc.get_page_resources(*page_id);

        let content = lopdf::content::Content::decode(&page).unwrap();
        let extracted_text = content
            .operations
            .iter()
            .filter_map(|operation| match operation.operator.as_ref() {
                "Tj" | "TJ" => Some(
                    operation
                        .operands
                        .iter()
                        .map(|operand| extract_text_from_object(operand))
                        .collect::<Vec<_>>()
                        .concat(),
                ),
                _ => None,
            })
            .collect::<Vec<_>>()
            .concat();

        text.push_str(&extracted_text);
    }

    // Normalize the text
    let normalized_text = text.nfc().collect::<String>();

    // Replace ligature 'ﬀ' with 'ff'
    let corrected_text = normalized_text.replace("ﬀ", "ff");
    return content;
}

fn extract_text_from_object(object: &Object) -> String {
    match object {
        Object::String(ref bytes, _) => String::from_utf8_lossy(&bytes).into_owned(),
        Object::Array(ref array) => array
            .iter()
            .map(|obj| extract_text_from_object(obj))
            .collect::<Vec<_>>()
            .concat(),
        _ => String::new(),
    }
}
