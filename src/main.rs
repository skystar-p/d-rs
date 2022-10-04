use clap::Parser;

const DIC_URL: &str = "https://dic.daum.net/search.do";

mod db;
mod parse;

#[derive(Parser, Debug)]
struct Args {
    /// search terms. multiple terms will be concatenated with space.
    #[clap(required = true)]
    words: Vec<String>,

    /// remove the word from history
    #[clap(long)]
    forget: bool,

    /// review the history
    #[clap(long, conflicts_with = "forget", conflicts_with = "words")]
    review: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.review {
        let reviews = db::list_reviews()?;
        for review in reviews {
            println!("{}", review.word);
        }
        return Ok(());
    }

    let word = args.words.join(" ");

    if args.forget {
        forget_word(&word)?;
        return Ok(());
    }

    search_word(&word).await?;

    Ok(())
}

async fn search_word(word: &str) -> anyhow::Result<()> {
    let raw_html = reqwest::get(format!("{}?q={}", DIC_URL, word))
        .await?
        .text()
        .await?;

    let meanings = parse::parse_meaning(&raw_html)?;

    println!("{}", meanings.join(" / "));

    db::save_history(&word, false)?;

    Ok(())
}

fn forget_word(word: &str) -> anyhow::Result<()> {
    db::remove_history(word)?;
    println!("word \"{}\" is removed from history", word);
    Ok(())
}
