const DIC_URL: &str = "https://dic.daum.net/search.do";

mod parse;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        println!("Usage: {} <word>", args[0]);
        return Ok(());
    }

    let word = args[1..].join(" ");

    let raw_html = reqwest::get(format!("{}?q={}", DIC_URL, word))
        .await?
        .text()
        .await?;

    let meanings = parse::parse_meaning(&raw_html)?;

    println!("{}", meanings.join(" / "));

    Ok(())
}
