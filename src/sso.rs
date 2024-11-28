use crate::Config;
use buaa_api::Session;

pub async fn login(session: &Session, config: &Config) {
    if !config.is_valid() {
        println!("[Error]::<SSO>: Please set username and password first");
        return;
    }
    let username = &config.username;
    let password = &config.password;
    match session.sso_login(username, password).await {
        Ok(_) => println!("[Info]::<SSO>: Login successfully"),
        Err(e) => eprintln!("[Info]::<SSO>: Login failed: {}", e),
    };
}
