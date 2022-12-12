use std::env;
use teloxide::{prelude::*, net::Download};
use tokio::fs::File;
use clap::Parser;
use once_cell::sync::Lazy;
use log::*;

async fn handler(message:Message, bot: Bot) -> Result<(), teloxide::RequestError>{

    if let Some(id) = ARGS.allowed_user.clone() {
        if message.chat.id.to_string() != id {
            bot.send_message(message.chat.id, "Permission Denied.").await.expect("Unable to send message.");
            return Ok(())
        }
    }

    if let Some(v) = message.video() {
        info!("Download Request {} received. Source chat: {}", v.file_id.clone(), message.chat.id.to_string());
        let video_file = bot.get_file(v.file_id.clone()).send().await;
        if let Ok(f) = video_file {
            debug!("File accquired");
            let file_path = ARGS.path.clone() + "/" + v.file_name.clone().unwrap_or(f.file_id).as_str();
            debug!("Saving to {}", file_path);
            let mut file = File::create(file_path.clone()).await.unwrap();
            debug!("File created, initiating download");
            bot.download_file(&f.file_path, &mut file).await.expect("Unable to download file.");
            info!("Download Complete, file saved to {}", file_path);
            bot.send_message(message.chat.id, "Download Complete").await.expect("Unable to send message.");
        } else {
            error!("{}",video_file.unwrap_err().to_string())
        }
        return Ok(());
    }
    info!("No video received.");
    Ok(())
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long, value_parser)]
    token: String,
    #[clap(short, long, value_parser, default_value = ".")]
    path: String,
    /// Loglevel: debug, info, warn, error, default = info
    #[clap(short, long, value_parser, default_value = "info")]
    loglevel: String,
    #[clap(short, long, value_parser)]
    allowed_user: Option<String>
}

static  ARGS: Lazy<Args> = Lazy::new(|| Args::parse());


#[tokio::main]
async fn main() {
    

    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", ARGS.loglevel.as_str());
    }
    pretty_env_logger::init();

    info!("Logger Initalized.");
    let bot = Bot::from_env();
    info!("Bot Configured.");
    warn!("TGBot-Downloader started. Target Directory: {}", ARGS.path.clone());
    if let Some(s) = ARGS.allowed_user.clone() {
        warn!("Whitelist enabled: {}", s)
    }
    teloxide::repl(bot, handler).await;
}