mod boya;
mod class;
mod command;
mod tes;
mod utils;
mod wifi;

use buaa_api::Context;
use clap::Parser;

use command::{Boya, BoyaSub, Class, ClassSub, Cli, Commands, Tes, TesSub, Wifi, WifiSub};

#[tokio::main]
async fn main() {
    let path = utils::get_path("./").unwrap();
    let context = Context::with_auth(&path);

    let cli = Cli::parse();

    match cli.command {
        Commands::Login { username, password } => {
            if let Some(un) = username {
                context.set_username(&un);
            };
            if let Some(pw) = password {
                context.set_password(&pw);
            }
            match context.sso().login().await {
                Ok(_) => println!("[Info]::<SSO>: Login successfully"),
                Err(e) => eprintln!("[Info]::<SSO>: Login failed: {e}"),
            };
        }
        Commands::Boya(Boya { command }) => match command {
            BoyaSub::Query { all } => {
                boya::query(&context, all).await;
            }
            BoyaSub::Rule { id } => {
                boya::rule(&context, id).await;
            }
            BoyaSub::Select { id } => {
                boya::choose(&context, id).await;
            }
            BoyaSub::Drop { id } => {
                boya::drop(&context, id).await;
            }
            BoyaSub::Check { id } => {
                boya::check(&context, id).await;
            }
            BoyaSub::Status { selected } => {
                boya::status(&context, selected).await;
            }
        },
        Commands::Class(Class { command }) => match command {
            ClassSub::Auto => {
                class::auto(&context).await;
            }
            ClassSub::Query { id } => {
                class::query(&context, id).await;
            }
            ClassSub::Checkin { id } => {
                class::checkin(&context, &id).await;
            }
        },
        Commands::Tes(Tes { command }) => match command {
            TesSub::Auto => {
                tes::auto(&context).await;
            }
            TesSub::List { all } => {
                tes::list(&context, all).await;
            }
        },
        Commands::Wifi(Wifi { command }) => match command {
            WifiSub::Login => {
                wifi::login(&context).await;
            }
            WifiSub::Logout => {
                wifi::logout(&context).await;
            }
        },
    }
    context.save_auth(&path);
}
