use buaa_api::Context;
use buaa_api::api::class::Course;
use time::format_description;
use tokio::time::Duration;

use std::fs::OpenOptions;

use crate::utils;

pub async fn auto(context: &Context) {
    let class = context.class();

    let time = utils::get_datetime();
    let format = format_description::parse("[year][month][day]").unwrap();
    let date = time.format(&format).unwrap();

    let schedule = match class.query_schedule(&date).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[Error]::<Smart Classroom>: Query schedule failed: {}", e);
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
                let rand = utils::simple_rand_range(5, 240);
                let wait = second as u64 + rand;
                println!("[Info]::<Smart Classroom>: Waiting for {} seconds", wait);
                tokio::time::sleep(Duration::from_secs(wait)).await;
            }
            checkin(&context, &s.id).await;
        }
    }
    println!("[Info]::<Smart Classroom>: Auto checkin finished");
}

pub async fn query(context: &Context, id: Option<String>) {
    let class = context.class();
    let path = crate::utils::get_path("class-schedule.json").unwrap();
    match id {
        Some(id) => {
            match id.len() {
                // Course ID
                5 => {
                    let schedules = match class.query_course_schedule(&id).await {
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
                    crate::utils::print_table(builder);
                }
                // Date. Format: YYYYMMDD
                8 => {
                    let schedule = match class.query_schedule(&id).await {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("[Error]::<Smart Classroom>: Query course failed: {}", e);
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
                    crate::utils::print_table(builder);
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
            crate::utils::print_table(builder);
        }
    };
}

pub async fn checkin(context: &Context, id: &str) {
    let class = context.class();
    let id_type = id.len();
    match id_type {
        // Schedule ID
        7 => {
            match class.checkin(id).await {
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
