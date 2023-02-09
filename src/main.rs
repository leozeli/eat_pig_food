use clap::{Parser, error};
use log::*;
use std::{env, error::Error, path};
use teloxide::{
    net::Download,
    prelude::*,
    utils::command::{self, BotCommands},
    RequestError,
};
use tokio::fs::File;

#[macro_use]
extern crate lazy_static;

async fn command_handler(msg: Message, bot: Bot, cmd: Command) -> Result<(),RequestError> {
    match cmd {
        Command::DownloadChannel{channelid} => download_handler(msg, bot).await?,
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }

    };
    Ok(())
}


async fn download_handler(message: Message, bot: Bot) -> Result<(), RequestError> {
    if let Some(id) = ARGS.allowed_user.clone() {
        if message.chat.id.to_string() != id {
            bot.send_message(message.chat.id, "Permission Denied.")
                .await
                .expect("Unable to send message.");
            return Ok(());
        }
    }

    if let Some(v) = message.video() {
        info!(
            "Download Request {} received. Source chat: {}",
            v.file.id.clone(),
            message.chat.id.to_string()
        );
        let video_file = bot.get_file(v.file.id.clone()).send().await;
        if let Ok(f) = video_file {
            debug!("File accquired");
            let file_path =
                ARGS.path.clone() + "/" + v.file_name.clone().unwrap_or(f.id.to_string()).as_str();
            debug!("Saving to {}", file_path);
            let mut file = File::create(file_path.clone()).await.unwrap();
            debug!("File created, initiating download");
            bot.download_file(&f.path, &mut file)
                .await
                .expect("Unable to download file.");
            info!("Download Complete, file saved to {}", file_path);
            bot.send_message(message.chat.id, "Download Complete")
                .await
                .expect("Unable to send message.");
        } else {
            error!("{}", video_file.unwrap_err().to_string())
        }
        return Ok(());
    }
    info!("No video received.");
    Ok(())
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long, value_parser, env = "TELOXIDE_TOKEN")]
    token: String,
    #[clap(short, long, value_parser, default_value = ".")]
    path: String,
    /// Loglevel: debug, info, warn, error, default = info
    #[clap(short, long, value_parser, default_value = "info")]
    loglevel: String,
    #[clap(short, long, value_parser, default_value = "573167966")]
    allowed_user: Option<String>,
}

#[derive(BotCommands, Clone)]
#[command(description = "Commands:", rename_rule = "lowercase")]
enum Command {
    #[command(description = "channel id to download")]
    DownloadChannel { channelid: String },
    #[command(description = "show this text")]
    Help,
}

lazy_static! {
    static ref ARGS: Args = Args::parse();
}

#[tokio::main]
async fn main() {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", ARGS.loglevel.as_str());
    }
    pretty_env_logger::init();

    info!("Logger Initalized.");
    let bot = Bot::from_env();
    info!("Bot Configured.");
    warn!(
        "TGBot-Downloader started. Target Directory: {}",
        ARGS.path.clone()
    );
    if let Some(s) = ARGS.allowed_user.clone() {
        warn!("Whitelist enabled: {}", s)
    }
    teloxide::repl(bot, handler).await;
}
