use crate::Config;
use buaa_api::Session;

pub async fn login(session: &Session, config: &Config) {
    if !config.is_valid() {
        println!("[Error]::<WIFI>: Please use `buaa login` to set username and password first");
        return;
    }
    let username = &config.username;
    let password = &config.password;
    match session.wifi_login(username, password).await {
        Ok(_) => println!(
            "[Info]::<WIFI>: Login successfully, Please wait a few seconds for the server to respond"
        ),
        Err(e) => eprintln!("[Info]::<WIFI>: Login failed: {}", e),
    };
}
