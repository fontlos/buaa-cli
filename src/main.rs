mod boya;
mod class;
mod command;
mod sso;
mod util;
mod wifi;

use buaa_api::Context;
use clap::Parser;

use command::{Boya, BoyaSub, Class, ClassSub, Cli, Commands};

#[tokio::main]
async fn main() {
    let cookie = util::get_path("buaa-cookie.json").unwrap();
    let config = crate::util::get_path("buaa-config.json").unwrap();
    let context = Context::new();
    context.with_config(&config);
    context.with_cookies(&cookie);

    let cli = Cli::parse();

    match cli.command {
        Commands::Login { username, password } => {
            if let Some(un) = username {
                context.set_username(&un);
            };
            if let Some(pw) = password {
                context.set_password(&pw);
            }
            sso::login(&context).await;
        }
        Commands::Boya(Boya { command }) => match command {
            BoyaSub::Login => {
                boya::login(&context).await;
            }
            BoyaSub::Query { all } => {
                boya::query(&context, all).await;
            }
            BoyaSub::Select { id } => {
                boya::choose(&context, id).await;
            }
            BoyaSub::Drop { id } => {
                boya::drop(&context, id).await;
            }
            BoyaSub::Status { selected } => {
                boya::status(&context, selected).await;
            }
        },
        Commands::Class(Class { command }) => match command {
            ClassSub::Login => {
                class::login(&context).await;
            }
            ClassSub::Auto => {
                class::auto(&context).await;
            }
            ClassSub::Query { id } => {
                class::query(&context, id).await;
            }
            ClassSub::Checkin { id, time } => {
                class::checkin(&context, id, time).await;
            }
        },
        Commands::Wifi => {
            wifi::login(&context).await;
        }
    }
    context.save();
    context.save_config(&config);
}
