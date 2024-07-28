use html_parser::Dom;
use scraper::{Html, Selector};
use std::{collections::HashMap, fs, path::Iter};

//This should iterate through the string then tokenise each part
// And bark the tokens out
struct Lexer<'a> {
    content: &'a [char], // Will store the pointer to the current starting letter of the string to the end
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
    let search = "cdk layout";
    let dir_path = "/Users/athul/Programming/rsearch/files";
    let file_dir = fs::read_dir(dir_path).unwrap();

    let mut file_words_map: DocWordsFreq = HashMap::new();

    //Indexing Files
    for file in file_dir {
        let file_path = file.unwrap().path();
        let file_string = fs::read_to_string(file_path.clone()).unwrap();

        let document = Html::parse_document(&file_string);

        // Create a selector for all text nodes
        let selector = Selector::parse("body").expect("Failed to create selector");

        let mut html_extracted_content = String::new();
        // Extract and print all text content
        for element in document.select(&selector) {
            let text = element.text().collect::<Vec<_>>().join(" ");
            html_extracted_content = html_extracted_content + &text
        }

        let char_arr = html_extracted_content.chars().collect::<Vec<char>>();
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
        let path_as_string = file_path.to_str().unwrap().to_string();
        file_words_map.insert(path_as_string, words_count);
        
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
            let tf = calculate_tf(&foo, term_freq) * calculate_idf(&foo, &file_words_map);
            total_tf += tf;
        }

        rank.insert(doc.to_string(), total_tf);
    }

    

    // Convert the HashMap to a Vec of tuples
    let mut sorted: Vec<_> = rank.into_iter().collect();

    // Sort the Vec by value (second element of the tuple)
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

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
