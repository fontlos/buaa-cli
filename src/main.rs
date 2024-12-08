mod boya;
mod class;
mod command;
mod config;
mod sso;
mod util;
mod wifi;

use buaa_api::Session;
use clap::Parser;

use command::{Boya, BoyaSub, Class, ClassSub, Cli, Commands};
use config::Config;

#[tokio::main]
async fn main() {
    let cookie = util::get_path("buaa-cookie.json").unwrap();
    let mut session = Session::new_in_file(cookie.to_str().unwrap());
    let mut config = Config::new();

    let cli = Cli::parse();

    match cli.command {
        Commands::Login { username, password } => {
            if let Some(un) = username {
                config.username = un;
            }
            if let Some(pw) = password {
                config.password = pw;
            }
            sso::login(&session, &config).await;
        }
        Commands::Boya(Boya { command }) => match command {
            BoyaSub::Login => {
                boya::login(&session, &mut config).await;
            }
            BoyaSub::Query { all } => {
                boya::query(&session, &mut config, all).await;
            }
            BoyaSub::Select { id } => {
                boya::choose(&session, &config, id).await;
            }
            BoyaSub::Drop { id } => {
                boya::drop(&session, &config, id).await;
            }
            BoyaSub::Status { selected } => {
                boya::status(&session, &mut config, selected).await;
            }
        },
        Commands::Class(Class { command }) => match command {
            ClassSub::Login => {
                class::login(&session, &mut config).await;
            }
            ClassSub::Auto => {
                class::auto(&session).await;
            }
            ClassSub::Query { id } => {
                class::query(&session, &config.class_token, id).await;
            }
            ClassSub::Checkin { id, time } => {
                class::checkin(&session, &config.class_token, id, time).await;
            }
        },
        Commands::Wifi => {
            wifi::login(&session, &config).await;
        }
    }
    session.save();
}
