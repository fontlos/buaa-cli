use buaa_api::{Context, Error};
use time::Date;
use tokio::time::Duration;

use std::io::Write;

pub async fn login(context: &Context) {
    let boya = context.boya();
    match boya.login().await {
        Ok(()) => {
            println!("[Info]::<Boya>: Login successfully");
        }
        Err(e) => {
            if let Error::LoginExpired(_) = e {
                println!("[Info]::<Boya>: Try refresh SSO token");
                match context.login().await {
                    Ok(_) => {
                        println!("[Info]::<Boya>: SSO refresh successfully");
                        match boya.login().await {
                            Ok(()) => {
                                println!("[Info]::<Boya>: Login successfully");
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

pub async fn query(context: &Context, all: bool) {
    let boya = context.boya();
    let courses = match boya.query_course().await {
        Ok(courses) => courses,
        Err(e) => {
            if let Error::LoginExpired(_) = e {
                println!("[Info]::<Boya>: Try refresh Boya token");
                login(context).await;
                match boya.query_course().await {
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
                course.selected
                    || (course.capacity.current < course.capacity.max
                        && course.time.select_end > time)
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
        tokio::time::sleep(duration).await;
        // 可能 token 已经过期重新获取一下
        match boya.login().await {
            Ok(()) => {
                println!("[Info]::<Boya>: Refresh token successfully");
            }
            Err(e) => {
                eprintln!("[Info]::<Boya>: Refresh token failed: {}", e);
                return;
            }
        };
    }

    // 之前少等待了10秒, 现在计算还需等待多久
    let now = buaa_api::utils::get_primitive_time();
    let duration = course.time.select_start - now;
    let second = duration.whole_seconds();
    if second > 0 {
        let duration = Duration::from_secs(second as u64);
        tokio::time::sleep(duration).await;
    }

    choose(context, id).await;
}

pub async fn choose(context: &Context, id: u32) {
    let boya = context.boya();
    let retry = 20;
    let retry_interval = Duration::from_millis(250);
    let mut interval = tokio::time::interval(retry_interval);
    for i in 0..retry {
        match boya.select_course(id).await {
            Ok(_) => {
                println!("[Info]::<Boya>: Select successfully");
                return;
            }
            Err(e) => {
                if i == retry - 1 {
                    eprintln!("[Error]::<Boya>: Select failed: {}", e);
                    return;
                }
                println!(
                    "[Info]::<Boya>: Select failed: {}. Retry {} times",
                    e,
                    i + 1
                );
            }
        }
        interval.tick().await; // 等待0.25秒
    }
}

pub async fn drop(context: &Context, id: u32) {
    let boya = context.boya();
    match boya.drop_course(id).await {
        Ok(_) => {
            println!("[Info]::<Boya>: Drop successfully");
        }
        Err(e) => {
            eprintln!("[Error]::<Boya>: Drop failed: {}", e);
            eprintln!("[Info]::<Boya>: Consider login again");
        }
    }
}

pub async fn status(context: &Context, selected: bool) {
    let now = buaa_api::utils::get_primitive_time();
    let middle = Date::from_calendar_date(now.year(), time::Month::July, 1).unwrap();
    let now_date = Date::from_calendar_date(now.year(), now.month(), now.day()).unwrap();
    let (start, end) = if now_date < middle {
        // 上半年
        (
            Date::from_calendar_date(now.year(), time::Month::February, 1).unwrap(),
            Date::from_calendar_date(now.year(), time::Month::July, 1).unwrap(),
        )
    } else {
        // 下半年
        (
            Date::from_calendar_date(now.year(), time::Month::August, 1).unwrap(),
            Date::from_calendar_date(now.year() + 1, time::Month::January, 1).unwrap(),
        )
    };
    let boya = context.boya();
    if selected {
        match boya.query_selected(start, end).await {
            Ok(s) => {
                println!("[Info]::<Boya>: Selected courses:");
                println!("{}", s)
            }
            Err(e) => {
                if let Error::LoginExpired(_) = e {
                    println!("[Info]::<Boya>: Try refresh Boya token");
                    login(context).await;
                    match boya.query_selected(start, end).await {
                        Ok(c) => {
                            println!("[Info]::<Boya>: Selected courses:");
                            println!("{}", c)
                        }
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
        match boya.query_statistic().await {
            Ok(s) => {
                println!("[Info]::<Boya>: Statistic information:");
                println!("{}", s)
            }
            Err(e) => {
                if let Error::LoginExpired(_) = e {
                    println!("[Info]::<Boya>: Try refresh Boya token");
                    login(context).await;
                    match boya.query_statistic().await {
                        Ok(s) => {
                            println!("[Info]::<Boya>: Statistic information:");
                            println!("{}", s)
                        }
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
