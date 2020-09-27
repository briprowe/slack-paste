use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

use clap::{App, Arg, SubCommand};
use directories::ProjectDirs;
use read_input::prelude::*;
use serde::{Deserialize, Serialize};
use slack_morphism::api::*;
use slack_morphism::*;
use slack_morphism_models::blocks::{SlackBlockMarkDownText, SlackSectionBlock};
use slack_morphism_models::*;
use toml;

type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Serialize, Deserialize)]
struct Config {
    slack_token: String,
}

fn render_message(content: &str) -> SlackMessageContent {
    SlackMessageContent::new().with_blocks(slack_blocks![some_into(
        SlackSectionBlock::new().with_text(md!("```{}```", content))
    )])
}

async fn post_message<T: Into<SlackChannelId>>(
    config: Config,
    destination: T,
) -> Result<(), Error> {
    let mut buffer = String::new();

    io::stdin().read_to_string(&mut buffer)?;

    let slack = SlackClient::new();

    let token_value = SlackApiTokenValue::from(config.slack_token);
    let token = SlackApiToken::new(token_value);

    let session = slack.open_session(&token);

    let post_chat_req =
        SlackApiChatPostMessageRequest::new(destination.into(), render_message(&buffer));

    session.chat_post_message(&post_chat_req).await?;

    Ok(())
}

async fn init(config_file: &Path) -> Result<(), Error> {
    let config_dir = config_file
        .parent()
        .ok_or(format!("No parent for: {}", config_file.to_string_lossy()))?;

    std::fs::create_dir_all(config_dir)?;
    let mut f = File::create(config_file)?;
    let slack_token: String = input().msg("Please input the slack token: ").get();
    f.write_all(toml::to_string_pretty(&Config { slack_token })?.as_bytes())?;

    println!("Config written to: {}", config_file.to_string_lossy());

    Ok(())
}

fn read_config(config_file: &Path) -> Result<Config, Error> {
    let mut buffer = String::new();
    let mut config = File::open(config_file)?;
    config.read_to_string(&mut buffer)?;

    Ok(toml::from_str(&buffer)?)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let app_dir = ProjectDirs::from("com", "briprowe", "slack-paste")
        .ok_or("Couldn't find a configuration directory")?;
    let default_config_file = app_dir.config_dir().join("config.toml");

    let app = App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!("\n"))
        .about(clap::crate_description!())
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("The location of the config file.")
                .default_value(
                    default_config_file
                        .to_str()
                        .expect("Invalid default config filename."),
                ),
        )
        .subcommand(
            SubCommand::with_name("paste")
                .about("Paste the contents of stdin to slack")
                .arg(
                    Arg::with_name("DESTINATION")
                        .help("The slack user or channel that should receive the paste.")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(SubCommand::with_name("init").about("Initialize slack app credentials"));

    let matches = app.clone().get_matches();

    let config_file = matches
        .value_of("config")
        .map(std::path::Path::new)
        .ok_or("Invalid config file path")?;

    match matches.subcommand_name() {
        Some("init") => init(config_file).await,
        Some("paste") => {
            let config = read_config(config_file)?;
            let destination = matches
                .subcommand_matches("paste")
                .ok_or("WTF")?
                .value_of("DESTINATION")
                .ok_or("No destination!")?;
            post_message(config, destination).await
        }
        Some(unknown) => Err(format!("Unknown command: {}", unknown))?,
        None => {
            let mut stderr = std::io::stderr();
            app.write_help(&mut stderr)?;
            std::process::exit(-1);
        }
    }
}
