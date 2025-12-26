use buaa_api::Context;
use buaa_api::crypto::rand::{Rng, WyRng};
use time::format_description;
use tokio::time::Duration;

use crate::utils;

pub async fn auto(context: &Context) {
    let time = utils::get_datetime();
    let format = format_description::parse("[year][month][day]").unwrap();
    let date = time.format(&format).unwrap();

    checkin_date(context, &date).await;
    println!("[Info]::<Smart Classroom>: Auto checkin finished");
}

pub async fn query(context: &Context, id: String) {
    let class = context.class();
    match id.len() {
        // Course ID
        5 => {
            let schedules = match class.query_course_schedule(&id).await {
                Ok(schedule) => schedule,
                Err(e) => {
                    eprintln!("[Error]::<Smart Classroom>: Query schedule failed: {e}");
                    return;
                }
            };
            let mut builder = tabled::builder::Builder::new();
            builder.push_record(["ID", "Time", "Status"]);
            for s in schedules {
                builder.push_record([&s.id, &s.time.to_string(), &s.status.to_string()]);
            }
            crate::utils::print_table(builder);
        }
        // Date. Format: YYYYMMDD
        8 => {
            let schedule = match class.query_schedule(&id).await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[Error]::<Smart Classroom>: Query course failed: {e}");
                    return;
                }
            };
            let mut builder = tabled::builder::Builder::new();
            builder.push_record(["Course ID", "ID", "Course", "Teacher", "Time", "Status"]);
            for c in schedule {
                builder.push_record([
                    &c.course_id,
                    &c.id,
                    &c.name,
                    &c.teacher,
                    &c.time.to_string(),
                    &c.status.to_string(),
                ]);
            }
            crate::utils::print_table(builder);
        }
        // Term ID
        9 => {
            let courses = match class.query_course(&id).await {
                Ok(courses) => courses,
                Err(e) => {
                    eprintln!("[Error]::<Smart Classroom>: Query course failed: {e}");
                    return;
                }
            };
            let mut builder = tabled::builder::Builder::new();
            builder.push_record(["ID", "Course", "Teacher"]);
            for c in courses {
                builder.push_record([&c.id, &c.name, &c.teacher]);
            }
            crate::utils::print_table(builder);
        }
        _ => {
            println!("[Error]::<Smart Classroom>: Invalid ID");
        }
    }
}

pub async fn checkin(context: &Context, id: &str) {
    match id.len() {
        // Schedule ID
        7 => {
            checkin_schedule(context, id).await;
        }
        // Date
        8 => {
            checkin_date(context, id).await;
        }
        _ => {
            println!("[Error]::<Smart Classroom>: Invalid ID");
        }
    }
}

pub async fn checkin_date(context: &Context, date: &str) {
    let class = context.class();

    let schedule = match class.query_schedule(date).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[Error]::<Smart Classroom>: Query schedule failed: {e}");
            return;
        }
    };

    for s in &schedule {
        if s.status == 0 {
            println!(
                "[Info]::<Smart Classroom>: Checkin for {} ID: {}",
                s.name, s.id
            );
            let now = utils::get_datetime();
            // 距离签到开始的时间, 上课前十分钟
            let duration = s.time - now;
            let second = duration.whole_seconds() - 600;
            // 如果已经开始签到就不等待了直接签到
            if second > 0 {
                // 如果是预签到, 我们尽可能早一点, 但加上随机扰动, 模拟人类行为
                // 考虑到准时可能导致失败, 我们加上一个 5 到 240 秒的随机扰动
                let mut rng = WyRng::new();
                let rand = rng.random_range(5u8..=240);
                let wait = second as u64 + rand as u64;
                println!("[Info]::<Smart Classroom>: Waiting for {wait} seconds");
                tokio::time::sleep(Duration::from_secs(wait)).await;
            }
            checkin_schedule(context, &s.id).await;
        }
    }
}

pub async fn checkin_schedule(context: &Context, id: &str) {
    let class = context.class();
    match class.checkin(id).await {
        Ok(_) => {
            println!("[Info]::<Smart Classroom>: Checkin successfully");
        }
        Err(e) => {
            eprintln!("[Error]::<Smart Classroom>: Checkin failed: {e}");
        }
    }
}
