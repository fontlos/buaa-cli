mod boya;
mod class;
mod command;
mod tes;
mod sso;
mod util;
mod wifi;

use buaa_api::Context;
use clap::Parser;

use command::{
    Boya, BoyaSub, Class, ClassSub, Cli, Commands, Tes, TesSub, Wifi, WifiSub,
};

#[tokio::main]
async fn main() {
    let context = Context::with_auth("./");

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
        Commands::Tes(Tes { command }) => match command {
            TesSub::Auto => {
                tes::auto(&context).await;
            }
            TesSub::List { all } => {
                tes::list(&context, all).await;
            }
            TesSub::Fill => {
                tes::fill(&context).await;
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
    context.save_auth("./");
}
