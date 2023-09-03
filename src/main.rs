use {
    reqwest,
    serde_derive::{Deserialize, Serialize},
    std::collections::HashMap,
    std::env,
    std::fs,
    std::io,
    std::path::{Path, PathBuf},
    std::process::Command,
    structopt::StructOpt,
    url::Url,
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ret {
    pub images: Vec<Image>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Image {
    pub url: String,
    pub copyright: String,
    pub title: String,
}

const BING: &str = "http://cn.bing.com";
const BING_IMAGE_REQUEST_URL: &str = "http://www.bing.com/HPImageArchive.aspx";
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36";
const SAVE_PATH: &str = "wallpaper";

#[derive(StructOpt, Debug)]
#[structopt(name = "ring")]
struct Args {
    #[structopt(short, long, default_value = SAVE_PATH)]
    path: String,
}

fn main() {
    let opt = Args::from_args();
    let url = get_image_url();

    match Some(url) {
        Some(i) => download(opt.path, i),
        None => println!("Parse picture url failed"),
    }
}

fn get_image_url() -> String {
    let query_parsms = &[
        ("format", "js"),
        ("idx", "0"),
        ("n", "1"),
        ("uhd", "1"),
        ("uhdwidth", "3840"),
        ("uhdheight", "2160"),
    ];

    let client = reqwest::blocking::Client::new();
    let resp = match client
        .get(BING_IMAGE_REQUEST_URL)
        .query(query_parsms)
        .header("User-Agent", USER_AGENT)
        .header("Referer", BING)
        .send()
    {
        Ok(resp) => resp.text().unwrap(),
        Err(err) => panic!("Error: {}", err),
    };

    let ret: Ret = serde_json::from_str(&resp).unwrap();

    ret.images[0].url.to_string()
}

fn download(path: String, uri: String) {
    let mut url: String = BING.to_owned();
    url.push_str(&uri);

    let parsed_url = Url::parse(&url).unwrap();
    let hash_query: HashMap<_, _> = parsed_url.query_pairs().into_owned().collect();

    let id = match hash_query.get("id") {
        Some(id) => id.to_string(),
        None => {
            panic!("There is no picture id")
        }
    };

    set_picture(path, url, id)
}

fn set_picture(path: String, url: String, id: String) {
    let save_path = match get_save_path(path) {
        Ok(p) => p,
        _ => panic!(),
    };

    match fs::create_dir_all(&save_path) {
        Err(why) => {
            println!("Failed to create directory! {:?}", why.to_string());
        }
        Ok(_) => {}
    }

    let fname = save_path.join(id);
    let mut file = std::fs::File::create(&fname).unwrap();
    reqwest::blocking::get(url)
        .unwrap()
        .copy_to(&mut file)
        .unwrap();

    let file_path = &fname.as_path().to_str().unwrap();

    let cmd_arg: String = format!(
        r#"tell application "Finder" to set desktop picture to POSIX file "{}""#,
        file_path
    );

    let cmd = Command::new("osascript").args(&["-e", &cmd_arg]).output();
    match cmd {
        Err(error) => {
            println!("Failed to set wallpaper:{:?}", error)
        }
        Ok(_) => println!("Set wallpaper successfully"),
    }
}

pub fn get_save_path(path: impl AsRef<Path>) -> io::Result<PathBuf> {
    let path = path.as_ref();

    let save_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()?.join(path)
    };

    Ok(save_path)
}
