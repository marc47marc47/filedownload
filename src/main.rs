use reqwest;
use select::document::Document;
use select::predicate::Name;
use std::fs::File;
use std::io::Write;
use tokio;
use futures_util::StreamExt;
use percent_encoding::percent_decode_str;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let url = "https://example.com"; // 替換為您的目標網頁
    let url = "https://tool.kametwu.com:8443/oracle/Doc/";
    let response = reqwest::get(url).await?.text().await?;

    let document = Document::from(response.as_str());
    let pdf_links: Vec<String> = document.find(Name("a"))
        .filter_map(|n| n.attr("href"))
        .filter(|&link| link.ends_with(".pdf"))
        .map(|link| link.to_string())
        .collect();

    for (i, pdf_link) in pdf_links.iter().enumerate() {
        let pdf_url = if pdf_link.starts_with("http") {
            pdf_link.clone()
        } else {
            format!("{}{}", url, pdf_link)
        };

        let pdf_response = reqwest::get(&pdf_url).await?;
        let total_size = pdf_response.content_length().unwrap_or(0);
        //let mut pdf_file = File::create(pdf_link.split('/').last().unwrap())?;
        let mut downloaded: u64 = 0;
        let mut stream = pdf_response.bytes().await?;

        let filename = pdf_link.split('/').last().unwrap();
        let filename = percent_encoding::percent_decode_str(filename).decode_utf8()?;
        let mut pdf_file = File::create(filename.as_ref())?;
        pdf_file.write_all(&stream)?;
        downloaded += stream.len() as u64;
        print!("\rDownloading {}: {:.2}%", pdf_link, (downloaded as f64 / total_size as f64) * 100.0);
        std::io::stdout().flush()?;

        println!("\nDownloaded: {}", pdf_link);
    }

    Ok(())
}

