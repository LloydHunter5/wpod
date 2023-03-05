use std::{thread, time::Duration};
use reqwest::blocking::get as reqwest_get;
use clap::{Command,Arg, ArgAction};
use serde_json::Value;
use clokwerk::{Scheduler, TimeUnits, Job};


enum Source {
    NationalGeographic,
    NASA,
    OutdoorPhotographer
}

const API_KEY : &str = "Mr42XkjQaVIjw80ezQKAe7cs21JkVV8yV73UTvTI";

fn get_natgeo_img()-> String{
    let response_json = reqwest_get("https://www.natgeotv.com/ca/photo-of-the-day")
        .expect("Could not fetch webpage from \'https://www.natgeotv.com/ca/photo-of-the-day\'")
        .text()
        .unwrap();
    let document = scraper::Html::parse_document(&response_json);
    let wrapper_selector = scraper::Selector::parse("a#POD05.DisplayBlock").unwrap();
    let img_selector = scraper::Selector::parse("img").unwrap();
    let img_url = document.select(&wrapper_selector)
        .next()
        .unwrap()
        .select(&img_selector)
        .next()
        .unwrap()
        .value()
        .attr("src").unwrap();
    return img_url.to_string();
}

fn get_nasa_img(api_key : &str) -> String {
    let api_response = reqwest_get("https://api.nasa.gov/planetary/apod?api_key=".to_owned() + &api_key).expect("Invalid API key!");
    let img_json = api_response.text().unwrap();
    let img_json : Value = serde_json::from_str(&img_json).unwrap();
    let img_url = img_json.get("hdurl").unwrap();
    return img_url.to_string().trim_matches('\"').to_string();
}


//TODO: get higher quality image from site
fn get_outdoor_photographer_img() -> String{
    let response_json = reqwest_get("https://www.outdoorphotographer.com/blog/category/photo-of-the-day/")
        .unwrap()
        .text()
        .unwrap();

    let document =  scraper::Html::parse_document(&response_json);
    let target ="img.attachment-mdv-gallery-view.size-mdv-gallery-view.wp-post-image";
    let image_selector = scraper::Selector::parse(target).unwrap();
    let img_url = document.select(&image_selector).next().unwrap().value().attr("src").unwrap();
    return img_url.to_string();
}

fn cache_image(img_url : &str) -> String{
    let img_bytes = reqwest_get(img_url).unwrap().bytes().unwrap();
    let cache_dir = dirs::cache_dir().unwrap().to_str().unwrap().to_owned() + "temp_wallpaper_file.jpg";
    let cache_dir = cache_dir.as_str();
    std::fs::write(cache_dir, img_bytes).unwrap();
    return cache_dir.to_string();
}

fn set_wallpaper(source :&Source, api_key : &str){
    let img_url = match source {
        Source::NationalGeographic => get_natgeo_img(),
        Source::NASA => get_nasa_img(api_key),
        Source::OutdoorPhotographer => get_outdoor_photographer_img()
    };

    let image_path = cache_image(img_url.as_str());
    wallpaper::set_from_path(image_path.as_str()).unwrap();
}


fn main() {
    let mut source : Source = Source::NASA;
    let mut key = API_KEY;

    let args = Command::new("wallpaper-set")
            .version("0.3.0")
            .about("wallpaper grabs either the NASA APOD or the National Geographic picture of the day, and sets it as your desktop background")
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
    
    
    if args.get_flag("nasa"){
        source = Source::NASA;
    }else if args.get_flag("natgeo"){
        source = Source::NationalGeographic;
    }else if args.get_flag("outdoorphoto"){
        source = Source::OutdoorPhotographer;
    }

    if args.contains_id("apikey"){
        key = args.get_one::<&str>("apikey").expect("expected a valid API key!");
    }

    set_wallpaper(&source, key);

    //continues to run the app in the background, updating the image every 24 hours
    if args.get_flag("background"){
        let mut scheduler = Scheduler::with_tz(chrono::Utc);
        scheduler.every(1.day())
            .at("00:15").run(move || set_wallpaper(&source, key));
        loop{
            scheduler.run_pending();
            thread::sleep(Duration::from_secs(3600));
        }
    }
}
