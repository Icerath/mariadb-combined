mod format_url;
mod kb_urls;
mod logger;
mod scrape;
mod scrape_client;

use chrono::Utc;
use clap::{arg, Command};
use log::info;
use std::{fs, time::Duration};

use scrape::ScrapeMethod;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const _BASE_URL: &str = "https://mariadb.com/kb/";
const BASE_PATH: &str = "../html/";
const DEFAULT_WAIT_TIME: Duration = Duration::from_millis(500);
const URL_LOCATIONS_SEP: char = ' ';
const URL_LOCATIONS_PATH: &str = "../../url_locations.txt";
const CONNECTION_WAIT_TIME: Duration = Duration::from_secs(3);
const LAST_UPDATED_PATH: &str = "last_updated.txt";

fn main() {
    logger::init();
    let args = parse_args();
    if args.clear {
        println!("{:?}", clear_archive());
    }
    let scraper = scrape::Scraper::new(DEFAULT_WAIT_TIME, args.ignore_existing).unwrap();
    scraper.scrape(args.scrape_method);
    set_last_updated();
}

#[derive(Debug)]
struct AppArgs {
    scrape_method: ScrapeMethod,
    clear: bool,
    ignore_existing: bool,
}

fn parse_args() -> AppArgs {
    let matches = Command::new("KbScraper")
        .arg(arg!([scrape_method] "[standard|resume|recent|pdf|pdf_langs]"))
        .arg(arg!(-c --clear "Clears out the html directory"))
        .arg(arg!(-r --resume "Resumes the scrape ignoring already scraped directories"))
        .get_matches();

    let scrape_method_string = matches
        .get_one::<String>("scrape_method")
        .map_or_else(|| "standard", String::as_str);

    let scrape_method = match scrape_method_string.to_lowercase().as_str() {
        "" | "standard" => ScrapeMethod::Standard,
        "recent" => ScrapeMethod::RecentChanges,
        "pdf" => ScrapeMethod::Pdf,
        "pdf_langs" => ScrapeMethod::PdfLangs,
        other => panic!("Invalid Scrape Method: '{other}'"),
    };

    let clear = matches.get_one::<bool>("clear").copied().unwrap_or(false);
    let ignore_existing = matches.get_one::<bool>("resume").copied().unwrap_or(false);
    AppArgs {
        scrape_method,
        clear,
        ignore_existing,
    }
}

fn clear_archive() -> Result<()> {
    info!("Clearing Archive");
    fs::remove_dir_all(BASE_PATH)?;
    fs::remove_file(URL_LOCATIONS_PATH)?;
    fs::remove_file(LAST_UPDATED_PATH)?;
    Ok(())
}

fn set_last_updated() {
    let text = Utc::now().date_naive().to_string();
    fs::write(LAST_UPDATED_PATH, text).unwrap();
}
