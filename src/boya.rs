use buaa_api::Context;
use buaa_api::api::boya::{
    Campus, Capacity, Category, Course, Schedule, Selected, SignRule, Statistic,
};
use tokio::time::Duration;

use std::io::Write;

use crate::utils;

pub async fn query(context: &Context, all: bool) {
    let boya = context.boya();
    let courses = match boya.query_course().await {
        Ok(courses) => courses,
        Err(e) => {
            eprintln!("[Error]::<Boya>: Query failed: {}", e);
            return;
        }
    };
    // 默认显示过滤过的可选课程
    if all {
        print_course(courses.iter());
    } else {
        let time = utils::get_datetime();
        let courses = courses.iter().filter(|course| {
            course.selected
                || (course.capacity.current < course.capacity.max
                    && course.schedule.select_end > time)
        });

        print_course(courses);
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

    let course = match courses.iter().find(|course| course.id == id) {
        Some(course) => course,
        None => {
            eprintln!("[Error]::<Boya>: Course not found");
            return;
        }
    };
    let now = utils::get_datetime();
    let duration = course.schedule.select_start - now;
    let second = duration.whole_seconds();
    // 如果时间大于 10 那么就等待并提前十秒重置token, 否则直接选课
    if second > 10 {
        let duration = Duration::from_secs((second - 10) as u64);
        println!("[Info]::<Boya>: Waiting for {} seconds", second);
        tokio::time::sleep(duration).await;
        // 提前手动刷新 token
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
    let now = utils::get_datetime();
    let duration = course.schedule.select_start - now;
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
        }
    }
}

pub async fn rule(context: &Context, id: u32) {
    let boya = context.boya();
    match boya.query_sign_rule(id).await {
        Ok(rule) => match rule {
            Some(rule) => {
                print_sign_rule(&rule);
            }
            None => {
                println!("[Info]::<Boya>: This course does not support check-in/out");
            }
        },
        Err(e) => {
            eprintln!("[Error]::<Boya>: Query sign rule failed: {}", e);
        }
    }
}

pub async fn check(context: &Context, id: u32) {
    let boya = context.boya();
    let rule = match boya.query_sign_rule(id).await {
        Ok(rule) => match rule {
            Some(rule) => rule,
            None => {
                println!("[Info]::<Boya>: This course does not support check-in/out");
                return;
            }
        },
        Err(e) => {
            eprintln!("[Error]::<Boya>: Query sign rule failed: {}", e);
            return;
        }
    };
    if rule.checkin_start > utils::get_datetime() {
        println!("[Info]::<Boya>: Check-in has not started yet");
        return;
    }
    if rule.checkin_end > utils::get_datetime() {
        match boya.checkin_course(id, &rule.coordinate).await {
            Ok(_) => {
                println!("[Info]::<Boya>: Check-in successfully");
            }
            Err(e) => {
                eprintln!("[Error]::<Boya>: Check-in failed: {}", e);
            }
        }
        return;
    }
    if rule.checkout_start > utils::get_datetime() {
        println!("[Info]::<Boya>: Check-out has not started yet");
        return;
    }
    if rule.checkout_end > utils::get_datetime() {
        match boya.checkout_course(id, &rule.coordinate).await {
            Ok(_) => {
                println!("[Info]::<Boya>: Check-out successfully");
            }
            Err(e) => {
                eprintln!("[Error]::<Boya>: Check-out failed: {}", e);
            }
        }
        return;
    }
    println!("[Info]::<Boya>: Check-in/out time has passed");
}

pub async fn status(context: &Context, selected: bool) {
    let boya = context.boya();
    if selected {
        // 完全成功或失败就直接返回, 否则尝试刷新登陆状态
        match boya.query_selected(None).await {
            Ok(s) => {
                println!("[Info]::<Boya>: Selected courses:");
                print_selected(&s);
                return;
            }
            Err(e) => {
                eprintln!("[Error]::<Boya>: Query failed: {}", e);
                return;
            }
        }
    } else {
        match boya.query_statistic().await {
            Ok(s) => {
                println!("[Info]::<Boya>: Statistic information:");
                print_statistic(&s);
                return;
            }
            Err(e) => {
                eprintln!("[Error]::<Boya>: Query failed: {}", e);
                return;
            }
        }
    }
}

// ======================= Print Course =======================

fn tabled_name(s: &str) -> String {
    textwrap::wrap(s, 18).join("\n")
}

fn tabled_location(s: &str) -> String {
    textwrap::wrap(s, 15).join("\n")
}

fn tabled_schedule(time: &Schedule) -> String {
    let format_string =
        time::format_description::parse("[year].[month].[day] [hour]:[minute]").unwrap();

    let formatted_course_start = time.course_start.format(&format_string).unwrap();
    let formatted_course_end = time.course_end.format(&format_string).unwrap();
    let formatted_select_start = time.select_start.format(&format_string).unwrap();
    let formatted_select_end = time.select_end.format(&format_string).unwrap();

    format!(
        "             CourseTime\n{} - {}\n             SelectTime\n{} - {}",
        formatted_course_start, formatted_course_end, formatted_select_start, formatted_select_end
    )
}

fn tabled_category(category: &Category) -> String {
    match category {
        Category::Arts => "美育".to_string(),
        Category::Ethics => "德育".to_string(),
        Category::Labor => "劳动教育".to_string(),
        Category::Safety => "安全健康".to_string(),
        Category::Other => "其他".to_string(),
    }
}

fn tabled_capacity(capacity: &Capacity) -> String {
    format!("{} / {}", capacity.current, capacity.max)
}

fn tabled_campus(capacity: &Campus) -> String {
    match capacity {
        Campus::XueYuanLu => "学院路".to_string(),
        Campus::ShaHe => "沙河".to_string(),
        Campus::All => "全部".to_string(),
    }
}

fn print_course<'a, I>(data: I)
where
    I: Iterator<Item = &'a Course>,
{
    let mut builder = tabled::builder::Builder::new();
    builder.push_record([
        "ID", "Course", "Position", "Time", "Kind", "Capacity", "Campus", "State",
    ]);
    for c in data {
        builder.push_record([
            &c.id.to_string(),
            &tabled_name(&c.name),
            &tabled_location(&c.location),
            &tabled_schedule(&c.schedule),
            &tabled_category(&c.category),
            &tabled_capacity(&c.capacity),
            &tabled_campus(&c.campus),
            &c.selected.to_string(),
        ]);
    }
    crate::utils::print_table(builder);
}

// ======================= Print Selected =======================

fn print_selected(data: &Vec<Selected>) {
    let mut builder = tabled::builder::Builder::new();
    builder.push_record(["ID", "Course", "Position", "Time", "Kind"]);
    for c in data {
        builder.push_record([
            &c.id.to_string(),
            &tabled_name(&c.name),
            &tabled_location(&c.location),
            &tabled_schedule(&c.schedule),
            &tabled_category(&c.category),
        ]);
    }
    crate::utils::print_table(builder);
}

// ======================= Print Sign Rule =======================

fn print_sign_rule(data: &SignRule) {
    let mut builder = tabled::builder::Builder::new();
    builder.push_record([
        "Check-in Start",
        "Check-in End",
        "Check-out Start",
        "Check-out End",
        "Coordinate",
    ]);
    builder.push_record([
        &data.checkin_start.to_string(),
        &data.checkin_end.to_string(),
        &data.checkout_start.to_string(),
        &data.checkout_end.to_string(),
    ]);
    crate::utils::print_table(builder);
}

// ======================= Print Statistic =======================

fn print_statistic(data: &Statistic) {
    let mut builder = tabled::builder::Builder::new();
    builder.push_record(["Kind", "Require", "Select", "Complete", "Fail", "Undone"]);
    builder.push_record([
        &tabled_category(&Category::Ethics),
        &data.ethics.require.to_string(),
        &data.ethics.select.to_string(),
        &data.ethics.complete.to_string(),
        &data.ethics.fail.to_string(),
        &data.ethics.undone.to_string(),
    ]);
    builder.push_record([
        &tabled_category(&Category::Arts),
        &data.arts.require.to_string(),
        &data.arts.select.to_string(),
        &data.arts.complete.to_string(),
        &data.arts.fail.to_string(),
        &data.arts.undone.to_string(),
    ]);
    builder.push_record([
        &tabled_category(&Category::Labor),
        &data.labor.require.to_string(),
        &data.labor.select.to_string(),
        &data.labor.complete.to_string(),
        &data.labor.fail.to_string(),
        &data.labor.undone.to_string(),
    ]);
    builder.push_record([
        &tabled_category(&Category::Safety),
        &data.safety.require.to_string(),
        &data.safety.select.to_string(),
        &data.safety.complete.to_string(),
        &data.safety.fail.to_string(),
        &data.safety.undone.to_string(),
    ]);
    crate::utils::print_table(builder);
}
