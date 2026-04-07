use buaa_api::Context;
use buaa_api::api::boya::{
    Campus, Capacity, Category, Course, Schedule, Selected, Semester, Statistic,
};
use buaa_api::time::DateTime;
use tokio::time::Duration;

pub async fn query(context: &Context, all: bool, page: u8) {
    let boya = context.boya();
    let courses = match boya.query_courses(page, 10).await {
        Ok(courses) => courses,
        Err(e) => {
            eprintln!("[Error]::<Boya>: Query failed: {e}");
            return;
        }
    };
    // 默认显示过滤过的可选课程
    if all {
        print_course(courses.iter());
    } else {
        let time = DateTime::now();
        let courses = courses.iter().filter(|course| {
            course.selected
                || (course.capacity.current < course.capacity.max
                    && course.schedule.select_end > time)
        });

        print_course(courses);
    }
}

pub async fn select(context: &Context, id: u32) {
    let boya = context.boya();

    let course = match boya.query_course(id).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[Error]::<Boya>: No course found: {e}");
            return;
        }
    };

    if course.selected {
        println!("[Info]::<Boya>: You have already selected this course");
        return;
    }

    // 提前十秒准备
    let prepare = course.schedule.select_start - Duration::from_secs(10);
    let now = DateTime::now();

    // 如果预备时间未到, 就等待提前刷新 Token, 因为时效只有十分钟
    if prepare > now {
        let duration = prepare - now;
        println!("[Info]::<Boya>: Waiting for {} seconds", duration.as_secs());
        tokio::time::sleep(duration).await;
        match boya.login().await {
            Ok(()) => {
                println!("[Info]::<Boya>: Refresh token successfully");
            }
            Err(e) => {
                eprintln!("[Info]::<Boya>: Refresh token failed: {e}");
                return;
            }
        };
    }

    // 以防万一如果之前等待刷新过 Token, 我们这里再次计算需要多久
    let now = DateTime::now();
    if course.schedule.select_start > now {
        let duration = course.schedule.select_start - now;
        tokio::time::sleep(duration).await;
    }

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
                    eprintln!("[Error]::<Boya>: Select failed: {e}");
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
            eprintln!("[Error]::<Boya>: Drop failed: {e}");
        }
    }
}

// TODO: 需要随机时间扰动
pub async fn check(context: &Context, id: u32) {
    let boya = context.boya();
    let config = match boya.query_course(id).await {
        Ok(c) => match c.sign_config {
            Some(config) => config,
            None => {
                println!("[Info]::<Boya>: This course does not support check-in/out");
                return;
            }
        },
        Err(e) => {
            eprintln!("[Error]::<Boya>: Query sign config failed: {e}");
            return;
        }
    };
    let now = DateTime::now();
    // 检查是否签到时间已过
    if now < config.checkin_end {
        // 检查是否在签到时间内
        if now < config.checkin_start {
            let duration = config.checkin_start - now;
            println!(
                "[Info]::<Boya>: Waiting for {} seconds to check-in",
                duration.as_secs()
            );
            tokio::time::sleep(duration).await;
        }
        match boya.checkin_course(id, &config.coordinate).await {
            Ok(_) => {
                println!("[Info]::<Boya>: Check-in successfully");
            }
            Err(e) => {
                eprintln!("[Error]::<Boya>: Check-in failed: {e}");
            }
        }
    }

    println!("[Info]::<Boya>: Check-in has passed");

    // 检查是否签退时间已过
    if now < config.checkout_end {
        // 检查是否在签退时间内
        if now < config.checkout_start {
            let duration = config.checkout_start - now;
            println!(
                "[Info]::<Boya>: Waiting for {} seconds to check-out",
                duration.as_secs()
            );
            tokio::time::sleep(duration).await;
        }
        match boya.checkout_course(id, &config.coordinate).await {
            Ok(_) => {
                println!("[Info]::<Boya>: Check-out successfully");
            }
            Err(e) => {
                eprintln!("[Error]::<Boya>: Check-out failed: {e}");
            }
        }
    }

    println!("[Info]::<Boya>: Check-out has passed");
}

pub async fn selected(context: &Context) {
    let boya = context.boya();
    match boya.query_selected(Semester::estimated_current()).await {
        Ok(s) => {
            println!("[Info]::<Boya>: Selected courses:");
            print_selected(&s);
        }
        Err(e) => {
            eprintln!("[Error]::<Boya>: Query failed: {e}");
        }
    }
}

pub async fn status(context: &Context) {
    let boya = context.boya();
    match boya.query_statistic().await {
        Ok(s) => {
            println!("[Info]::<Boya>: Statistic information:");
            print_statistic(&s);
        }
        Err(e) => {
            eprintln!("[Error]::<Boya>: Query failed: {e}");
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
    let formatted_course_start = time.course_start.format_short();
    let formatted_course_end = time.course_end.format_short();
    let formatted_select_start = time.select_start.format_short();
    let formatted_select_end = time.select_end.format_short();

    format!(
        "   CourseTime\n{formatted_course_start}\n{formatted_course_end}\n   SelectTime\n{formatted_select_start}\n{formatted_select_end}"
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

fn tabled_campuses(campuses: &Vec<Campus>) -> String {
    let mut campus_names: Vec<&str> = Vec::new();
    for campus in campuses {
        campus_names.push(match campus {
            Campus::XueYuanLu => "学院路",
            Campus::ShaHe => "沙河",
            Campus::HangZhou => "杭州",
            Campus::All => "全部",
            Campus::Unknown => "未知",
        });
    }
    campus_names.join(", ")
}

fn print_course<'a, I>(data: I)
where
    I: Iterator<Item = &'a Course>,
{
    let mut builder = tabled::builder::Builder::new();
    builder.push_record([
        "ID",
        "Course",
        "Position",
        "Time",
        "Kind",
        "Capacity",
        "Campus",
        "IsSelected",
        "CanCheck",
    ]);
    for c in data {
        builder.push_record([
            &c.id.to_string(),
            &tabled_name(&c.name),
            &tabled_location(&c.location),
            &tabled_schedule(&c.schedule),
            &tabled_category(&c.category),
            &tabled_capacity(&c.capacity),
            &tabled_campuses(&c.campuses),
            &c.selected.to_string(),
            &c.sign_config.is_some().to_string(),
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

// ======================= Print Sign Config =======================

// fn print_sign_config(data: &SignConfig) {
//     let mut builder = tabled::builder::Builder::new();
//     builder.push_record([
//         "Check-in Start",
//         "Check-in End",
//         "Check-out Start",
//         "Check-out End",
//         "Coordinate",
//     ]);
//     builder.push_record([
//         &data.checkin_start.to_string(),
//         &data.checkin_end.to_string(),
//         &data.checkout_start.to_string(),
//         &data.checkout_end.to_string(),
//     ]);
//     crate::utils::print_table(builder);
// }

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
