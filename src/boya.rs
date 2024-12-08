use buaa_api::{Session, SessionError};
use tokio::time::{self, Duration};

use std::io::Write;

use crate::Config;

pub async fn login(session: &Session, config: &mut Config) {
    match session.boya_login().await {
        Ok(t) => {
            println!("[Info]::<Boya>: Login successfully");
            config.boya_token = t;
        }
        Err(e) => {
            if let SessionError::LoginExpired(_) = e {
                println!("[Info]::<Boya>: Try refresh SSO token");
                match session.sso_login(&config.username, &config.password).await {
                    Ok(_) => {
                        println!("[Info]::<Boya>: SSO refresh successfully");
                        match session.boya_login().await {
                            Ok(t) => {
                                println!("[Info]::<Boya>: Login successfully");
                                config.boya_token = t;
                            }
                            Err(e) => eprintln!("[Error]::<Boya>: Login failed: {}", e),
                        }
                    }
                    Err(e) => eprintln!("[Error]::<Boya>: SSO Login failed: {}", e),
                }
            } else {
                eprintln!("[Error]::<Boya>: Login failed: {}", e);
            }
        }
    }
}

pub async fn query(session: &Session, config: &mut Config, all: bool) {
    let courses = match session.boya_query_course(&config.boya_token).await {
        Ok(courses) => courses,
        Err(e) => {
            if let SessionError::LoginExpired(_) = e {
                println!("[Info]::<Boya>: Try refresh Boya token");
                login(session, config).await;
                match session.boya_query_course(&config.boya_token).await {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("[Error]::<Boya>: Query failed: {}", e);
                        return;
                    }
                }
            } else {
                eprintln!("[Error]::<Boya>: Query failed: {}", e);
                return;
            }
        }
    };
    // 默认显示过滤过的可选课程
    if all {
        println!("{}", courses);
    } else {
        let time = buaa_api::utils::get_primitive_time();
        let courses = courses
            .iter()
            .filter(|course| {
                course.selected || (course.capacity.current < course.capacity.max && course.time.select_end > time)
            })
            .collect::<Vec<_>>();
        println!("{}", buaa_api::utils::table(&courses));
    }
    // 输入 ID 选择课程
    print!("[Info]::<Boya>: Type ID to select course: ");
    std::io::stdout().flush().unwrap();
    let mut id = String::new();
    std::io::stdin().read_line(&mut id).unwrap();

    let id: u32 = match id.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            eprintln!("[Error]::<Boya>: Invalid ID");
            return;
        }
    };

    let course = courses.iter().find(|course| course.id == id).unwrap();
    let now = buaa_api::utils::get_primitive_time();
    let duration = course.time.select_start - now;
    let second = duration.whole_seconds();
    // 如果时间大于 10 那么就等待并提前十秒重置token, 否则直接选课
    if second > 10 {
        let duration = Duration::from_secs((second - 10) as u64);
        println!("[Info]::<Boya>: Waiting for {} seconds", second);
        time::sleep(duration).await;
        // 可能 token 已经过期重新获取一下
        let token = match session.boya_login().await {
            Ok(s) => {
                println!("[Info]::<Boya>: Refresh token successfully");
                s
            }
            Err(e) => {
                eprintln!("[Info]::<Boya>: Refresh token failed: {}", e);
                return;
            }
        };
        config.boya_token = token;
    }

    // 之前少等待了10秒, 现在计算还需等待多久
    let now = buaa_api::utils::get_primitive_time();
    let duration = course.time.select_start - now;
    let second = duration.whole_seconds();
    if second > 0 {
        let duration = Duration::from_secs(second as u64);
        time::sleep(duration).await;
    }

    choose(session, config, id).await;
}

pub async fn choose(session: &Session, config: &Config, id: u32) {
    let retry = 20;
    let retry_interval = Duration::from_millis(250);
    let mut interval = time::interval(retry_interval);
    for i in 0..retry {
        match session.boya_select_course(&config.boya_token, id).await {
            Ok(_) => {
                println!("[Info]::<Boya>: Select successfully");
                return;
            }
            Err(e) => {
                if i == retry - 1 {
                    eprintln!("[Error]::<Boya>: Select failed: {}", e);
                    return;
                }
                println!("[Info]::<Boya>: Select failed: {}. Retry {} times", e, i + 1);
            }
        }
        interval.tick().await; // 等待0.25秒
    }
}

pub async fn drop(session: &Session, config: &Config, id: u32) {
    match session.boya_drop_course(&config.boya_token, id).await {
        Ok(_) => {
            println!("[Info]::<Boya>: Drop successfully");
        }
        Err(e) => {
            eprintln!("[Error]::<Boya>: Drop failed: {}", e);
            eprintln!("[Info]::<Boya>: Consider login again");
        }
    }
}

pub async fn status(session: &Session, config: &mut Config, selected: bool) {
    if selected {
        match session.boya_query_selected(&config.boya_token).await {
            Ok(s) => {
                println!("[Info]::<Boya>: Selected courses:");
                println!("{}", s)
            }
            Err(e) => {
                if let SessionError::LoginExpired(_) = e {
                    println!("[Info]::<Boya>: Try refresh Boya token");
                    login(session, config).await;
                    match session.boya_query_selected(&config.boya_token).await {
                        Ok(c) => {
                            println!("[Info]::<Boya>: Selected courses:");
                            println!("{}", c)
                        },
                        Err(e) => {
                            eprintln!("[Error]::<Boya>: Query failed: {}", e);
                            return;
                        }
                    }
                } else {
                    eprintln!("[Error]::<Boya>: Query failed: {}", e);
                }
            }
        }
    } else {
        match session.boya_query_statistic(&config.boya_token).await {
            Ok(s) => {
                println!("[Info]::<Boya>: Statistic information:");
                println!("{}", s)
            }
            Err(e) => {
                if let SessionError::LoginExpired(_) = e {
                    println!("[Info]::<Boya>: Try refresh Boya token");
                    login(session, config).await;
                    match session.boya_query_statistic(&config.boya_token).await {
                        Ok(s) => {
                            println!("[Info]::<Boya>: Statistic information:");
                            println!("{}", s)
                        },
                        Err(e) => {
                            eprintln!("[Error]::<Boya>: Query failed: {}", e);
                            return;
                        }
                    }
                } else {
                    eprintln!("[Error]::<Boya>: Query failed: {}", e);
                }
            }
        }
    }
}
