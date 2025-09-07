use buaa_api::api::boya::{
    BoyaCampus, BoyaCapacity, BoyaCourse, BoyaKind, BoyaSelected, BoyaStatistic, BoyaTime,
};
use buaa_api::Context;
use time::Date;
use tokio::time::Duration;

use std::io::Write;

pub async fn login(context: &Context) {
    let boya = context.boya();
    // 尝试登录, 现在 SSO 可以自动刷新
    match boya.login().await {
        Ok(()) => {
            println!("[Info]::<Boya>: Login successfully");
            return;
        }
        Err(e) => {
            eprintln!("[Error]::<Boya>: Login failed: {}", e);
            return;
        }
    }
}

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
        let time = buaa_api::utils::get_datatime();
        let courses = courses.iter().filter(|course| {
            course.selected
                || (course.capacity.current < course.capacity.max && course.time.select_end > time)
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

    let course = courses.iter().find(|course| course.id == id).unwrap();
    let now = buaa_api::utils::get_datatime();
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
    let now = buaa_api::utils::get_datatime();
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
    let now = buaa_api::utils::get_datatime();
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
        // 完全成功或失败就直接返回, 否则尝试刷新登陆状态
        match boya.query_selected(start, end).await {
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

// ======================= Print BoyaCourse =======================

fn tabled_name(s: &str) -> String {
    textwrap::wrap(s, 18).join("\n")
}

fn tabled_position(s: &str) -> String {
    textwrap::wrap(s, 15).join("\n")
}

fn tabled_time(time: &BoyaTime) -> String {
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

fn tabled_kind(capacity: &BoyaKind) -> String {
    match capacity {
        BoyaKind::Arts => "美育".to_string(),
        BoyaKind::Ethics => "德育".to_string(),
        BoyaKind::Labor => "劳动教育".to_string(),
        BoyaKind::Safety => "安全健康".to_string(),
        BoyaKind::Other => "其他".to_string(),
    }
}

fn tabled_capacity(capacity: &BoyaCapacity) -> String {
    format!("{} / {}", capacity.current, capacity.max)
}

fn tabled_campus(capacity: &BoyaCampus) -> String {
    match capacity {
        BoyaCampus::XueYuanLu => "学院路".to_string(),
        BoyaCampus::ShaHe => "沙河".to_string(),
        BoyaCampus::All => "全部".to_string(),
        BoyaCampus::Other => "其他".to_string(),
    }
}

fn print_course<'a, I>(data: I)
where
    I: Iterator<Item = &'a BoyaCourse>,
{
    let mut builder = tabled::builder::Builder::new();
    builder.push_record([
        "ID", "Course", "Position", "Time", "Kind", "Capacity", "Campus", "State",
    ]);
    for c in data {
        builder.push_record([
            &c.id.to_string(),
            &tabled_name(&c.name),
            &tabled_position(&c.position),
            &tabled_time(&c.time),
            &tabled_kind(&c.kind),
            &tabled_capacity(&c.capacity),
            &tabled_campus(&c.campus),
            &c.selected.to_string(),
        ]);
    }
    crate::util::print_table(builder);
}

// ======================= Print BoyaSelected =======================

fn print_selected(data: &Vec<BoyaSelected>) {
    let mut builder = tabled::builder::Builder::new();
    builder.push_record(["ID", "Course", "Position", "Time", "Kind"]);
    for c in data {
        builder.push_record([
            &c.id.to_string(),
            &tabled_name(&c.name),
            &tabled_position(&c.position),
            &tabled_time(&c.time),
            &tabled_kind(&c.kind),
        ]);
    }
    crate::util::print_table(builder);
}

// ======================= Print BoyaStatistic =======================

fn print_statistic(data: &BoyaStatistic) {
    let mut builder = tabled::builder::Builder::new();
    builder.push_record(["Kind", "Require", "Select", "Complete", "Fail", "Undone"]);
    builder.push_record([
        &tabled_kind(&BoyaKind::Ethics),
        &data.ethics.require.to_string(),
        &data.ethics.select.to_string(),
        &data.ethics.complete.to_string(),
        &data.ethics.fail.to_string(),
        &data.ethics.undone.to_string(),
    ]);
    builder.push_record([
        &tabled_kind(&BoyaKind::Arts),
        &data.arts.require.to_string(),
        &data.arts.select.to_string(),
        &data.arts.complete.to_string(),
        &data.arts.fail.to_string(),
        &data.arts.undone.to_string(),
    ]);
    builder.push_record([
        &tabled_kind(&BoyaKind::Labor),
        &data.labor.require.to_string(),
        &data.labor.select.to_string(),
        &data.labor.complete.to_string(),
        &data.labor.fail.to_string(),
        &data.labor.undone.to_string(),
    ]);
    builder.push_record([
        &tabled_kind(&BoyaKind::Safety),
        &data.safety.require.to_string(),
        &data.safety.select.to_string(),
        &data.safety.complete.to_string(),
        &data.safety.fail.to_string(),
        &data.safety.undone.to_string(),
    ]);
    crate::util::print_table(builder);
}
