use buaa_api::Context;
use buaa_api::api::class::Course;
use time::{PrimitiveDateTime, Time};
use tokio::time::Duration;

use std::fs::OpenOptions;

pub async fn auto(context: &Context) {
    let spoc = context.spoc();
    let class = context.class();
    // 从 Spoc 获取今日课表
    let week = match spoc.get_week().await {
        Ok(w) => w,
        Err(e) => {
            eprintln!("[Error]: Spoc Get week failed: {}", e);
            return;
        }
    };
    let week_schedule = match spoc.get_week_schedule(&week).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("[Error]: Spoc Get week schedule failed: {}", e);
            return;
        }
    };
    let now = buaa_api::utils::get_datetime();
    let weekday = now.weekday();
    // 过滤掉已经开始的课程
    let today_schedule = week_schedule
        .iter()
        .filter(|schedule| schedule.weekday == weekday && schedule.time.end > now)
        .collect::<Vec<_>>();

    // 获取学期课表
    // 2024-20251 -> 202420251
    let term = week.term.replace("-", "");
    let term_schedule = match class.query_course(&term).await {
        Ok(ts) => ts,
        Err(e) => {
            eprintln!(
                "[Error]::<Smart Classroom>: Query term course failed: {}",
                e
            );
            return;
        }
    };
    // 先循环今日课表, 在学期课表中去查询对应的课程 ID
    for ts in today_schedule {
        for s in &term_schedule {
            if ts.class_id == s.class_id {
                let now = buaa_api::utils::get_datetime();
                let target = ts.time.start;
                let duration = target - now;
                let second = duration.whole_seconds();
                println!("[Info]::<Smart Classroom>: Checkin for {} ID: {}", s.name, s.id);
                checkin_delay(&context, &s.id, second).await;
                break;
            }
        }
    }
    println!("[Info]::<Smart Classroom>: Auto checkin finished");
}

pub async fn query(context: &Context, id: Option<String>) {
    let class = context.class();
    let path = crate::util::get_path("class-schedule.json").unwrap();
    match id {
        Some(id) => {
            match id.len() {
                // Course ID
                5 => {
                    let schedules = match class.query_schedule(&id).await {
                        Ok(schedule) => schedule,
                        Err(e) => {
                            eprintln!("[Error]::<Smart Classroom>: Query schedule failed: {}", e);
                            return;
                        }
                    };
                    let mut builder = tabled::builder::Builder::new();
                    builder.push_record(["ID", "Time", "Status"]);
                    for s in schedules {
                        builder.push_record([&s.id, &s.time.to_string(), &s.status.to_string()]);
                    }
                    crate::util::print_table(builder);
                }
                // Term ID
                9 => {
                    let courses = match class.query_course(&id).await {
                        Ok(courses) => courses,
                        Err(e) => {
                            eprintln!("[Error]::<Smart Classroom>: Query course failed: {}", e);
                            return;
                        }
                    };
                    let file = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(path)
                        .unwrap();
                    serde_json::to_writer(file, &courses).unwrap();
                    let mut builder = tabled::builder::Builder::new();
                    builder.push_record(["ID", "Course", "Teacher"]);
                    for c in courses {
                        builder.push_record([&c.id, &c.name, &c.teacher]);
                    }
                    crate::util::print_table(builder);
                }
                _ => {
                    println!("[Error]::<Smart Classroom>: Invalid ID");
                    return;
                }
            }
        }
        None => {
            if !path.exists() {
                println!(
                    "[Error]::<Smart Classroom>: No local data. Use `buaa class query <term>` first"
                );
                return;
            }
            let file = OpenOptions::new().read(true).open(path).unwrap();
            let courses: Vec<Course> = serde_json::from_reader(file).unwrap();

            let mut builder = tabled::builder::Builder::new();
            builder.push_record(["ID", "Course", "Teacher"]);
            for c in courses {
                builder.push_record([&c.id, &c.name, &c.teacher]);
            }
            crate::util::print_table(builder);
        }
    };
}

pub async fn checkin(context: &Context, id: String, time: Option<String>) {
    let class = context.class();
    let id_type = id.len();
    match id_type {
        // Course ID
        5 => {
            let second = if let Some(time) = time {
                parse_delay_second(time)
            } else {
                println!("[Info]::<Smart Classroom>: Please input time by `-t`");
                return;
            };
            checkin_delay(context, &id, second).await;
            return;
        }
        // Schedule ID
        7 => {
            match class.checkin(&id).await {
                Ok(_) => {
                    println!("[Info]::<Smart Classroom>: Checkin successfully");
                }
                Err(e) => {
                    eprintln!("[Error]::<Smart Classroom>: Checkin failed: {}", e);
                }
            }
            return;
        }
        _ => {
            println!("[Error]::<Smart Classroom>: Invalid ID");
            return;
        }
    }
}

/// 解析时间字符串, 返回延迟秒数
fn parse_delay_second(time: String) -> i64 {
    let hour = time[0..2].parse::<u8>().unwrap();
    let minute = time[2..4].parse::<u8>().unwrap();
    let time = Time::from_hms(hour, minute, 0).unwrap();
    let now = buaa_api::utils::get_datetime();
    let target = PrimitiveDateTime::new(now.date(), time);
    let duration = target - now;
    let second = duration.whole_seconds();
    second
}

/// 延迟签到, 在延迟秒数的基础上加 5 秒, 防止网络延迟
async fn checkin_delay(context: &Context, id: &str, second: i64) {
    let class = context.class();
    if second > 0 {
        println!("[Info]::<Smart Classroom>: Waiting for {} seconds", second);
        tokio::time::sleep(Duration::from_secs((second + 5) as u64)).await;
    }
    let schedules = match class.query_schedule(id).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[Error]::<Smart Classroom>: Query schedule failed: {:?}", e);
            return;
        }
    };
    let schedule = schedules.last().unwrap();
    match class.checkin(&schedule.id).await {
        Ok(_) => {
            println!("[Info]::<Smart Classroom>: Checkin successfully");
        }
        Err(e) => {
            eprintln!("[Error]::<Smart Classroom>: Checkin failed: {}", e);
        }
    }
}
