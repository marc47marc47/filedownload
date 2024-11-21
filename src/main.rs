use reqwest;
use select::document::Document;
use select::predicate::Name;
use std::fs::File;
use std::io::Write;
use tokio;
use futures_util::StreamExt;
use percent_encoding::percent_decode_str;
use std::env;
use clap::{Arg, App};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("File Downloader")
        .version("1.0")
        .author("Author Name <marc.xiao marc47marc47@gmail.com>")
        .about("Downloads files from a given URL")
        .arg(Arg::new("url")
            .short('r')
            .long("url")
            .value_name("URL")
            .default_value("https://tool.kametwu.com:8443/oracle/Doc/")
            .help("Sets the URL to download files from")
            .takes_value(true)
            .required(true))
        .arg(Arg::new("file_types")
            .short('t')
            .long("types")
            .value_name("FILE_TYPES")
            .default_value("pdf,doc,docx,ppt,pptx,xls,xlsx,txt")
            .help("Sets the file types to download")
            .takes_value(true)
            .required(true))
        .get_matches();

    let url = matches.value_of("url").unwrap();
    let file_types_str = matches.value_of("file_types").unwrap();
    let file_types: Vec<&str> = file_types_str.split(',').map(|s| s.trim()).collect();

    let response = reqwest::get(url).await?.text().await?;

    println!("Downloading files from: {}", url);
    println!("File types: {:#?}", file_types);
    let document = Document::from(response.as_str());
    let file_links: Vec<String> = document.find(Name("a"))
        .filter_map(|n| n.attr("href"))
        .filter(|&link| file_types.iter().any(|&ft| link.ends_with(ft)))
        .map(|link| link.to_string())
        .collect();

    println!("Found {} files to download", file_links.len());
    for (i, file_link) in file_links.iter().enumerate() {
        let file_url = if file_link.starts_with("http") {
            file_link.clone()
        } else {
            format!("{}{}", url, file_link)
        };
        //let file_response = reqwest::get(&file_url).await?;
        // 處理下載的文件
        let pdf_response = reqwest::get(&file_url).await?;
        let total_size = pdf_response.content_length().unwrap_or(0);
        //let mut pdf_file = File::create(file_link.split('/').last().unwrap())?;
        let mut downloaded: u64 = 0;
        let mut stream = pdf_response.bytes().await?;
        
        let filename = file_link.split('/').last().unwrap();
        let filename = percent_encoding::percent_decode_str(filename).decode_utf8()?;
        let mut pdf_file = File::create(filename.as_ref())?;
        pdf_file.write_all(&stream)?;
        downloaded += stream.len() as u64;
        print!("\rDownloading {}: {:.2}%", file_link, (downloaded as f64 / total_size as f64) * 100.0);
        std::io::stdout().flush()?;
        
        println!("\nDownloaded: {}", file_link);
    }

    Ok(())
}