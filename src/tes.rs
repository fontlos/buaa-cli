use buaa_api::Context;
use buaa_api::api::tes::{Answer, Task};

use std::io::Write;

fn print_warning() {
    let mut builder = tabled::builder::Builder::new();
    let warning = "Warning!: Due to the poor design of the evaluation system server,
using this may cause the evaluation button on the web page to become unclickable.
But don't worry, the evaluation data has been submitted correctly.
If you want to view the evaluation results on the web page,
you can remove the 'disabled' attribute of the button in the browser console,
and you'll be able to click it.
Or you might wait a little longer, and it may return to normal.";
    builder.push_record([warning]);
    crate::utils::print_table(builder);
}

pub async fn list(context: &Context, all: bool) {
    print_warning();
    let tes = context.tes();
    let tasks = match tes.get_task().await {
        Ok(list) => list,
        Err(e) => {
            eprintln!("[Error]::<TES>: Get task failed: {e}");
            return;
        }
    };

    let tasks = if all {
        tasks
    } else {
        tasks.into_iter().filter(|l| !l.state).collect::<Vec<_>>()
    };

    let mut builder = tabled::builder::Builder::new();
    builder.push_record(["Course", "Teacher", "State"]);
    for t in &tasks {
        builder.push_record([&t.course, &t.teacher, &t.state.to_string()]);
    }
    crate::utils::print_table(builder);

    print!("[Info]::<TES>: Type index to fill: ");
    std::io::stdout().flush().unwrap();
    let mut str = String::new();
    std::io::stdin().read_line(&mut str).unwrap();
    let index = match str.trim().parse::<usize>() {
        Ok(i) => i,
        Err(e) => {
            eprintln!("[Error]::<TES>: Invalid index: {e}");
            return;
        }
    };

    let task = match tasks.get(index) {
        Some(task) => task,
        None => {
            eprintln!("[Error]::<TES>: Index out of range");
            return;
        }
    };
    fill(context, task).await;
}

async fn fill(context: &Context, task: &Task) {
    println!("[Info]::<TES>: ======================= Manual fill start =======================");
    let tes = context.tes();

    println!(
        "[Info]::<TES>: Course: {}, Teacher: {}",
        task.course, task.teacher
    );
    println!("[Info]::<TES>: Option is score, type the index");
    let form = match tes.get_form(task).await {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[Error]::<TES>: Get form failed: {e}");
            return;
        }
    };
    let mut ans: Vec<Answer> = Vec::with_capacity(form.questions.len());
    for (i, q) in form.questions.iter().enumerate() {
        println!("[Info]::<TES>: {}. {}", i + 1, q.name);
        if q.is_choice {
            let mut builder = tabled::builder::Builder::new();
            builder.push_record(["A", "B", "C", "D"]);
            builder.push_record([
                &q.choices[0].score.to_string(),
                &q.choices[1].score.to_string(),
                &q.choices[2].score.to_string(),
                &q.choices[3].score.to_string(),
            ]);
            crate::utils::print_table(builder);
        }
        print!("[Info]::<TES>: Type answer: ");
        std::io::stdout().flush().unwrap();
        let mut str = String::new();
        std::io::stdin().read_line(&mut str).unwrap();
        if q.is_choice {
            let index = match str.trim() {
                "A" | "a" => 0,
                "B" | "b" => 1,
                "C" | "c" => 2,
                "D" | "d" => 3,
                _ => {
                    eprintln!("[Error]::<TES>: Invalid choice");
                    return;
                }
            };
            ans.push(Answer::Choice(index));
        } else {
            let str = str.trim();
            if str.is_empty() {
                ans.push(Answer::Completion(None));
            } else {
                ans.push(Answer::Completion(Some(str.to_string())));
            }
        }
    }
    let mut complete = form.fill(ans);

    if complete.is_perfect() {
        println!("[Info]::<TES>: Perfect score! Need reason, between 10 and 200 characters.");
        let mut str = String::new();
        std::io::stdin().read_line(&mut str).unwrap();
        complete.set_reason(str.trim().to_string()).unwrap();
    }

    if complete.is_unqualified() {
        println!("[Warning]::<TES>: Unqualified score! Need reason, between 10 and 200 characters.");
        let mut str = String::new();
        std::io::stdin().read_line(&mut str).unwrap();
        complete.set_reason(str.trim().to_string()).unwrap();
    }

    print!(
        "[Info]::<TES>: Finall score is {}. Press Enter to submit",
        complete.score()
    );
    std::io::stdout().flush().unwrap();
    let _ = std::io::stdin().read_line(&mut String::new()).unwrap();

    match tes.submit_form(complete).await {
        Ok(_) => println!("[Info]::<TES>: Submit successfully"),
        Err(e) => eprintln!("[Error]::<TES>: Submit failed: {e}"),
    }

    println!("[Info]::<TES>: ======================== Manual fill end ========================");
}

pub async fn auto(context: &Context) {
    print_warning();
    print!("Warning!!! Press Enter to continue");
    std::io::stdout().flush().unwrap();
    let _ = std::io::stdin().read_line(&mut String::new()).unwrap();

    println!("[Info]::<TES>: ======================= Auto fill start =======================");
    let tes = context.tes();

    let tasks = match tes.get_task().await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("[Error]::<TES>: Get task failed: {e}");
            return;
        }
    };

    for t in tasks {
        if t.state {
            continue;
        }
        println!(
            "[Info]::<TES>: Course: {}, Teacher: {}",
            t.course, t.teacher
        );
        let form = match tes.get_form(&t).await {
            Ok(f) => f,
            Err(e) => {
                eprintln!("[Error]::<TES>: Get form failed: {e}");
                continue;
            }
        };
        let complete = form.fill_default();
        println!("[Info]::<TES>: Finall score is {}", complete.score());
        match tes.submit_form(complete).await {
            Ok(_) => println!("[Info]::<TES>: Submit successfully"),
            Err(e) => eprintln!("[Error]::<TES>: Submit failed: {e}"),
        }
        println!("[Info]::<TES>: Wait 1 second to avoid too fast");
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    println!("[Info]::<TES>: ======================== Auto fill end ========================");
}
