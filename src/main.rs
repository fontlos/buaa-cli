mod boya;
mod class;
mod command;
mod tes;
mod utils;
mod wifi;

use buaa_api::exports::ContextBuilder;
use buaa_api::store::cookies::CookieStore;
use buaa_api::store::cred::CredentialStore;
use clap::Parser;

use command::{Boya, BoyaSub, Class, ClassSub, Cli, Commands, Tes, TesSub, Wifi, WifiSub};

#[tokio::main]
async fn main() {
    let path = utils::get_path("./").unwrap();
    let cookies_path = path.join("cookies.json");
    let cookies = match CookieStore::from_file(cookies_path) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("[Warning]: Failed to read cookies.json, use default");
            CookieStore::default()
        }
    };
    let cred_path = path.join("cred.json");
    let cred = match CredentialStore::from_file(cred_path) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("[Warning]: Failed to read cred.json, use default");
            CredentialStore::default()
        }
    };

    let cli = Cli::parse();

    let context = ContextBuilder::new()
        .cookies(cookies)
        .cred(cred)
        .tls(!cli.disable_tls)
        .build();

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
            BoyaSub::Query { all, page } => {
                boya::query(&context, all, page).await;
            }
            BoyaSub::Select { id } => {
                boya::select(&context, id).await;
            }
            BoyaSub::Drop { id } => {
                boya::drop(&context, id).await;
            }
            BoyaSub::Check { id } => {
                boya::check(&context, id).await;
            }
            BoyaSub::Selected => {
                boya::selected(&context).await;
            }
            BoyaSub::Status => {
                boya::status(&context).await;
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
    match context.save_auth(&path) {
        Ok(_) => {}
        Err(_) => {
            eprintln!("[Error]: Failed to save auth data");
        }
    };
}
