use clap::Parser;
use log::*;
use std::{default, env};
use teloxide::{
    net::Download,
    prelude::*,
    types::Chat,
    utils::command::{ BotCommands},
    RequestError, dispatching::{dialogue::{InMemStorage, self}, UpdateHandler},
};
use tokio::fs::File;
#[macro_use]
extern crate lazy_static;

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default)]
enum State { 
    #[default]
    Start,
    RevicedMessage,
    RevicedMessageChoice,
}

async fn command_handler(msg: Message, bot: Bot, cmd: Command) -> HandlerResult {
    match cmd {
        Command::Download => download_handler(msg, bot).await?,
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::DownloadChannel => {
            if let Some(source_msg) = msg.forward_from_chat() {
                println!("{}", source_msg.id);
            }
            msg
        }
    };
    Ok(())
}

async fn download_handler(message: Message, bot: Bot) -> Result<Message, RequestError> {
    if let Some(id) = ARGS.allowed_user.clone() {
        if message.chat.id.to_string() != id {
            bot.send_message(message.chat.id, "Permission Denied.")
                .await
                .expect("Unable to send message.");
            return Ok(message);
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
        return Ok(message);
    }
    info!("No video received.");
    Ok(message)
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
    #[command(description = "download video")]
    Download,
    #[command(description = "channel id to download")]
    DownloadChannel,
    #[command(description = "show this text")]
    Help,
    #[command(description = "cancel ")]
    Cancel,
}

lazy_static! {
    static ref ARGS: Args = Args::parse();
}
async fn cancel(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Cancel the dialogue").await?;
    dialogue.exit().await?;
    Ok(())
}

async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "invalid command, please use /help ").await?;
    Ok(())
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;
    let command_handler = teloxide::filter_command::<Command, _>().branch(
        case![State::Start]
        .branch(case![Command::Help].endpoint(command_handler))
    )
    .branch(case![Command::Cancel].endpoint(cancel));


    
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
    Dispatcher::builder(bot, schema())
    .dependencies(dptree::deps![InMemStorage::<State>::new()])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
    bot.set_my_commands(Command::bot_commands()).await.unwrap();
    Command::repl(bot, command_handler).await;
}
