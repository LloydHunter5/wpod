use clap::{Arg, ArgAction, Command};
use clokwerk::{Job, Scheduler, TimeUnits};
use reqwest::blocking::get as reqwest_get;
use serde_json::Value;
use std::{thread, time::Duration};

enum Source {
    NationalGeographic,
    NASA,
    OutdoorPhotographer,
}

const NASA_API_KEY: &str = "Mr42XkjQaVIjw80ezQKAe7cs21JkVV8yV73UTvTI";

// Grabs the url of the National Geographic photo of the day
// (there are several, the Canadian one is the only one I've found that is still updated)
fn get_natgeo_img() -> String {
    let response_json = reqwest_get("https://www.natgeotv.com/ca/photo-of-the-day")
        .expect("Could not fetch webpage from \'https://www.natgeotv.com/ca/photo-of-the-day\'")
        .text()
        .unwrap();
    let document = scraper::Html::parse_document(&response_json);
    let wrapper_selector = scraper::Selector::parse(".DisplayBlock").unwrap();
    let img_selector = scraper::Selector::parse("img").unwrap();
    let img_url = document.select(&wrapper_selector)
        .next()
        .expect("Could not find the expected \"a\" tag. Did the page structure change?")
        .select(&img_selector)
        .next()
        .expect("Could not find the \"img\" tag within the provided context. Did the page structure change?")
        .value()
        .attr("src")
        .expect("Could not find the attribute tag \"src\" in the provided webpage. Did the structure of the page change?");
    return img_url.to_string();
}

// Grabs the url of the NASA APOD via the NASA APOD API using the provided API key
fn get_nasa_img(api_key: &str) -> String {
    let api_response =
        reqwest_get("https://api.nasa.gov/planetary/apod?api_key=".to_owned() + &api_key)
            .expect("Invalid API key!");
    let img_json = api_response.text().unwrap();
    let img_json: Value =
        serde_json::from_str(&img_json).expect("Failed to parse JSON from the NASA API get");
    let img_url = img_json.get("hdurl").unwrap();
    return img_url.to_string().trim_matches('\"').to_string();
}

fn get_alt_nasa_img(api_key: &str) -> String {
    let api_response =
        reqwest_get("https://api.nasa.gov/planetary/apod?api_key=".to_owned() + &api_key)
            .expect("Invalid API key!");
    let img_json = api_response.text().unwrap();
    let img_json: Value =
        serde_json::from_str(&img_json).expect("Failed to parse JSON from the NASA API get");
    let img_url = img_json.get("url").unwrap();
    return img_url.to_string().trim_matches('\"').to_string();
}

//TODO: get higher quality image from site
fn get_outdoor_photographer_img() -> String {
    let response_json =
        reqwest_get("https://www.outdoorphotographer.com/blog/category/photo-of-the-day/")
            .unwrap()
            .text()
            .unwrap();

    let document = scraper::Html::parse_document(&response_json);
    let target = "img.attachment-mdv-gallery-view.size-mdv-gallery-view.wp-post-image";
    let image_selector = scraper::Selector::parse(target).unwrap();
    let img_url = document
        .select(&image_selector)
        .next()
        .unwrap()
        .value()
        .attr("src")
        .unwrap();
    return img_url.to_string();
}

// Downloads the current image from the provided url, and caches it on the users computer. Returns the directory of the image
fn cache_image(source: &Source, img_url: &str, api_key: &str) -> String {
    let mut img_bytes = reqwest_get(img_url).unwrap().bytes().unwrap();
    if img_bytes.len() < 1000 && matches!(Source::NASA, source) {
        img_bytes = reqwest_get(get_alt_nasa_img(api_key))
            .unwrap()
            .bytes()
            .unwrap();
    }
    let cache_dir =
        dirs::cache_dir().unwrap().to_str().unwrap().to_owned() + "temp_wallpaper_file.jpg";
    let cache_dir = cache_dir.as_str();
    std::fs::write(cache_dir, img_bytes).unwrap();
    return cache_dir.to_string();
}

// sets the user's wallpaper to the image at the provided source
fn set_wallpaper(source: &Source, api_key: &str) {
    let img_url = match source {
        Source::NationalGeographic => get_natgeo_img(),
        Source::NASA => get_nasa_img(api_key),
        Source::OutdoorPhotographer => get_outdoor_photographer_img(),
    };

    let image_path = cache_image(source, img_url.as_str(), api_key);
    wallpaper::set_from_path(image_path.as_str()).expect("Wallpaper was not set successfully");
}

fn main() {
    let mut source: Source = Source::NASA;
    let mut key = NASA_API_KEY;

    let args = Command::new("wallpaper-set")
            .version("0.3.4")
            .about("wallpaper of the day grabs a \'photo of the day\' from a selection of websites, and automatically sets it to your desktop background")
            .args(&[
                Arg::new("nasa")
                .short('n')
                .long("nasa")
                .action(ArgAction::SetTrue),

                Arg::new("natgeo")
                .short('g')
                .long("natgeo")
                .action(ArgAction::SetTrue),

                Arg::new("outdoorphoto")
                .short('o')
                .long("outdoorphoto")
                .action(ArgAction::SetTrue),

                Arg::new("apikey")
                .short('k')
                .long("apikey")
                .action(ArgAction::Set),

                Arg::new("background")
                .short('b')
                .long("background")
                .action(ArgAction::SetTrue)

            ]
            ).get_matches();

    if args.get_flag("nasa") {
        source = Source::NASA;
    } else if args.get_flag("natgeo") {
        source = Source::NationalGeographic;
    } else if args.get_flag("outdoorphoto") {
        source = Source::OutdoorPhotographer;
    }

    if args.contains_id("apikey") {
        key = args
            .get_one::<&str>("apikey")
            .expect("expected a valid API key!");
    }

    set_wallpaper(&source, key);

    //continues to run the app in the background, updating the image every 24 hours at 00:15 UTC
    if args.get_flag("background") {
        let mut scheduler = Scheduler::with_tz(chrono::Utc);
        scheduler
            .every(1.day())
            .at("00:15")
            .run(move || set_wallpaper(&source, key));

        loop {
            scheduler.run_pending();
            thread::sleep(Duration::from_secs(3600));
        }
    }
}
