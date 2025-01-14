mod boya;
mod class;
mod command;
mod evaluation;
mod sso;
mod util;
mod wifi;

use buaa_api::Context;
use clap::Parser;

use command::{
    Boya, BoyaSub, Class, ClassSub, Cli, Commands, Evaluation, EvaluationSub, Wifi, WifiSub,
};

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
        Commands::Evaluation(Evaluation { command }) => match command {
            EvaluationSub::Auto => {
                evaluation::auto(&context).await;
            }
            EvaluationSub::List { all } => {
                evaluation::list(&context, all).await;
            }
            EvaluationSub::Fill => {
                evaluation::fill(&context).await;
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
    context.save_cookie(&cookie);
    context.save_config(&config);
}
