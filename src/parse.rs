use scraper::{Html, Selector};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("parse failed: {0}")]
    ParseError(String),
    #[error("no meaning found")]
    MeaningsNotFound,
}

pub fn parse_meaning(html: &str) -> Result<Vec<String>, Error> {
    let doc = Html::parse_document(html);
    let list_selector = Selector::parse("ul.list_search").unwrap();
    let li_selector = Selector::parse("li").unwrap();

    let ul = doc
        .select(&list_selector)
        .next()
        .ok_or_else(|| Error::ParseError("failed to find meaning list".to_string()))?;

    let mut meanings = vec![];
    let txt_selector = Selector::parse("span.txt_search").unwrap();
    for li in ul.select(&li_selector) {
        let span = li
            .select(&txt_selector)
            .next()
            .ok_or_else(|| Error::ParseError("failed to find meaning text".to_string()))?;
        meanings.push(span.text().collect());
    }

    if meanings.is_empty() {
        Err(Error::MeaningsNotFound)
    } else {
        Ok(meanings)
    }
}
