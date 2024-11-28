use buaa_api::{ClassCourse, Session};
use time::{PrimitiveDateTime, Time};
use tokio::time::Duration;

use std::fs::OpenOptions;

use crate::Config;

pub async fn login(session: &Session, config: &mut Config) {
    match session.class_login().await {
        Ok(t) => {
            println!("[Info]::<Smart Classroom>: Login successfully");
            config.class_token = t;
        }
        Err(e) => {
            eprintln!("[Error]::<Smart Classroom>: Login failed: {}", e);
        }
    }
}

pub async fn auto(session: &Session) {
    // 从 Spoc 获取今日课表
    let token = match session.spoc_login().await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("[Error]: Spoc Login failed: {}", e);
            return;
        }
    };
    let week = match session.spoc_get_week(&token).await {
        Ok(w) => w,
        Err(e) => {
            eprintln!("[Error]: Spoc Get week failed: {}", e);
            return;
        }
    };
    let week_schedule = match session.spoc_get_week_schedule(&token, &week).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("[Error]: Spoc Get week schedule failed: {}", e);
            return;
        }
    };
    let now = buaa_api::utils::get_primitive_time();
    let weekday = now.weekday();
    // 过滤掉已经开始的课程
    let today_schedule = week_schedule
        .iter()
        .filter(|schedule| schedule.weekday == weekday && schedule.time.end > now)
        .collect::<Vec<_>>();

    // 获取学期课表
    let token = match session.class_login().await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("[Error]::<Smart Classroom>: Login failed: {}", e);
            return;
        }
    };
    // 2024-20251 -> 202420251
    let term = week.term.replace("-", "");
    let term_schedule = match session.class_query_course(&token, &term).await {
        Ok(ts) => ts,
        Err(e) => {
            eprintln!("[Error]::<Smart Classroom>: Query term course failed: {}", e);
            return;
        }
    };
    // 先循环今日课表, 在学期课表中去查询对应的课程 ID
    for ts in today_schedule {
        for s in &term_schedule {
            if ts.name == s.name {
                let now = buaa_api::utils::get_primitive_time();
                let target = ts.time.start;
                let duration = target - now;
                let second = duration.whole_seconds();
                println!("[Info]::<Smart Classroom>: Checkin for {}", s.name);
                checkin_delay(&session, &token, &s.id, second).await;
                break;
            }
        }
    }
    println!("[Info]::<Smart Classroom>: Auto checkin finished");
}

pub async fn query(session: &Session, token: &str, id: Option<String>) {
    let path = crate::util::get_path("buaa-data-schedule.json").unwrap();
    match id {
        Some(id) => {
            match id.len() {
                // Course ID
                5 => {
                    let s = match session.class_query_schedule(&token, &id).await {
                        Ok(schedule) => schedule,
                        Err(e) => {
                            eprintln!("[Error]::<Smart Classroom>: Query schedule failed: {}", e);
                            println!("[Info]::<Smart Classroom>: Consider use `buaa class login` first");
                            return
                        }
                    };
                    println!("{}", buaa_api::utils::table(&s));
                },
                // Term ID
                9 => {
                    let c = match session.class_query_course(&token, &id).await {
                        Ok(courses) => courses,
                        Err(e) => {
                            eprintln!("[Error]::<Smart Classroom>: Query course failed: {}", e);
                            println!("[Info]::<Smart Classroom>: Consider use `buaa class login` first");
                            return
                        }
                    };
                    let file = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(path)
                        .unwrap();
                    serde_json::to_writer(file, &c).unwrap();
                    println!("{}", buaa_api::utils::table(&c));
                },
                _ => {
                    println!("[Error]::<Smart Classroom>: Invalid ID");
                    return
                }
            }
        },
        None => {
            if !path.exists() {
                println!("[Error]::<Smart Classroom>: No local data. Use `buaa class query <term>` first");
                return
            }
            let file = OpenOptions::new()
                .read(true)
                .open(path)
                .unwrap();
            let courses: Vec<ClassCourse> = serde_json::from_reader(file).unwrap();
            println!("{}", buaa_api::utils::table(&courses));
        }
    };
}

pub async fn checkin(session: &Session, token: &str, id: String, time: Option<String>) {
    let id_type = id.len();
    match id_type {
        // Course ID
        5 => {
            let second = if let Some(time) = time {
                parse_delay_second(time)
            } else {
                println!("[Info]::<Smart Classroom>: Please input time by `-t`");
                return
            };
            checkin_delay(session, &token, &id, second).await;
            return;
        }
        // Schedule ID
        7 => {
            match session.class_checkin(&token, &id).await {
                Ok(_) => {
                    println!("[Info]::<Smart Classroom>: Checkin successfully");
                }
                Err(e) => {
                    eprintln!("[Error]::<Smart Classroom>: Checkin failed: {}", e);
                }
            }
            return
        }
        _ => {
            println!("[Error]::<Smart Classroom>: Invalid ID");
            return
        }
    }
}

/// 解析时间字符串, 返回延迟秒数
fn parse_delay_second(time: String) -> i64 {
    let hour = time[0..2].parse::<u8>().unwrap();
    let minute = time[2..4].parse::<u8>().unwrap();
    let time = Time::from_hms(hour, minute, 0).unwrap();
    let now = buaa_api::utils::get_primitive_time();
    let target = PrimitiveDateTime::new(now.date(), time);
    let duration = target - now;
    let second = duration.whole_seconds();
    second
}

/// 延迟签到, 在延迟秒数的基础上加 5 秒, 防止网络延迟
async fn checkin_delay(session: &Session, token: &str, id: &str, second: i64) {
    if second > 0 {
        println!("[Info]::<Smart Classroom>: Waiting for {} seconds", second);
        tokio::time::sleep(Duration::from_secs((second + 5) as u64)).await;
    }
    let schedule = match session.class_query_schedule(token, id).await {
        Ok(schedule) => schedule,
        Err(e) => {
            eprintln!("[Error]::<Smart Classroom>: Query schedule failed: {:?}", e);
            return;
        }
    };
    let schedule = schedule.last().unwrap();
    match session.class_checkin(token, &schedule.id).await {
        Ok(_) => {
            println!("[Info]::<Smart Classroom>: Checkin successfully");
        }
        Err(e) => {
            eprintln!("[Error]::<Smart Classroom>: Checkin failed: {}", e);
        }
    }
}
