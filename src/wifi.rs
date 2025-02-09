use buaa_api::Context;

pub async fn login(context: &Context) {
    let wifi = context.wifi();
    match wifi.login().await {
        Ok(_) => println!(
            "[Info]::<WIFI>: Login successfully, Please wait a few seconds for the server to respond"
        ),
        Err(e) => eprintln!("[Info]::<WIFI>: Login failed: {}", e),
    };
}

pub async fn logout(context: &Context) {
    let wifi = context.wifi();
    match wifi.logout().await {
        Ok(_) => println!("[Info]::<WIFI>: Logout successfully"),
        Err(e) => eprintln!("[Info]::<WIFI>: Logout failed: {}", e),
    };
}
