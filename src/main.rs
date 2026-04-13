mod tui;
use tui::App;

mod config;
mod rss_def;

use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use toml;

#[tokio::main]
async fn main() {
    let config_path = match dirs::home_dir().map(|mut path| {
        path.push(".config");
        path.push("oxide");
        path.push("oxide.toml");
        path
    }) {
        Some(x) => x,
        None => {
            eprintln!("Oxide Error: Something went wrong");
            return;
        }
    };
    let file = match File::open(config_path) {
        Ok(x) => x,
        Err(_) => {
            eprintln!("Oxide Error: Can't find ~/.config/oxide/oxide.toml");
            return;
        }
    };
    let mut buf_reader = BufReader::new(file);
    let mut config_content = String::new();
    match buf_reader.read_to_string(&mut config_content) {
        Ok(_) => (),
        Err(_) => {
            eprintln!("Oxide Error: Can't read config file");
            return;
        }
    };

    let oxide_config: config::Config = match toml::from_str(config_content.as_str()) {
        Ok(x) => x,
        Err(_) => {
            eprintln!("Oxide Error: Can't parse the config file");
            return;
        }
    };

    let mut terminal = ratatui::init();
    let mut app = App::new(oxide_config);
    let res = app.run(&mut terminal).await;
    ratatui::restore();
    if let Err(e) = res {
        eprintln!("Oxide Error: {:?}", e);
    }
}
