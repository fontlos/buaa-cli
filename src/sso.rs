use buaa_api::Context;

pub async fn login(context: &Context) {
    match context.login().await {
        Ok(_) => println!("[Info]::<SSO>: Login successfully"),
        Err(e) => eprintln!("[Info]::<SSO>: Login failed: {}", e),
    };
}
