use buaa_api::api::tes::{EvaluationAnswer, EvaluationListItem};
use buaa_api::{Context, Error};

use std::io::Write;

pub async fn login(context: &Context) {
    let tes = context.tes();
    // 尝试登录, 如果是登录过期, 就继续执行, 其他错误就直接返回
    match tes.login().await {
        Ok(()) => {
            println!("[Info]::<TES>: Login successfully");
            return;
        }
        Err(Error::LoginExpired(_)) => println!("[Info]::<TES>: Try refresh SSO token"),
        Err(e) => {
            eprintln!("[Error]::<TES>: Login failed: {}", e);
            return;
        }
    }
    // 如果是登录过期就继续执行到这里, 尝试登录 SSO, 失败了就直接返回
    match context.login().await {
        Ok(_) => println!("[Info]::<TES>: SSO refresh successfully"),
        Err(e) => {
            eprintln!("[Error]::<TES>: SSO Login failed: {}", e);
            return;
        }
    }
    // SSO 登录成功, 尝试登录 Boya, 失败了就直接返回
    match tes.login().await {
        Ok(()) => println!("[Info]::<Boya>: Login successfully"),
        Err(e) => eprintln!("[Error]::<Boya>: Login failed: {}", e),
    }
}

pub async fn list(context: &Context, all: bool) {
    login(context).await;

    let tes = context.tes();
    let list = match tes.get_evaluation_list().await {
        Ok(list) => list,
        Err(e) => {
            eprintln!("[Error]::<TES>: Get list failed: {}", e);
            return;
        }
    };

    let list = if all {
        list
    } else {
        list.into_iter().filter(|l| !l.state).collect::<Vec<_>>()
    };

    let mut builder = tabled::builder::Builder::new();
    builder.push_record(["Course", "Teacher", "State"]);
    for l in &list {
        builder.push_record([&l.course, &l.teacher, &l.state.to_string()]);
    }
    crate::util::print_table(builder);

    print!("[Info]::<TES>: Type index to fill: ");
    std::io::stdout().flush().unwrap();
    let mut str = String::new();
    std::io::stdin().read_line(&mut str).unwrap();
    let index = match str.trim().parse::<usize>() {
        Ok(i) => i,
        Err(e) => {
            eprintln!("[Error]::<TES>: Invalid index: {}", e);
            return;
        }
    };

    let l = match list.get(index) {
        Some(l) => l,
        None => {
            eprintln!("[Error]::<TES>: Index out of range");
            return;
        }
    };

    println!(
        "[Info]::<TES>: Course: {}, Teacher: {}",
        l.course, l.teacher
    );
    println!("[Info]::<TES>: Option is score, type the index");
    let form = match tes.get_evaluation_form(&l).await {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[Error]::<TES>: Get form failed: {}", e);
            return;
        }
    };
    let mut ans: Vec<EvaluationAnswer> = Vec::with_capacity(form.questions.len());
    for (i, q) in form.questions.iter().enumerate() {
        println!("[Info]::<TES>: {}. {}", i + 1, q.name);
        if q.is_choice {
            let mut builder = tabled::builder::Builder::new();
            builder.push_record(["A", "B", "C", "D"]);
            builder.push_record([
                &q.options[0].score.to_string(),
                &q.options[1].score.to_string(),
                &q.options[2].score.to_string(),
                &q.options[3].score.to_string(),
            ]);
            crate::util::print_table(builder);
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
            ans.push(EvaluationAnswer::Choice(index));
        } else {
            ans.push(EvaluationAnswer::Completion(str.trim().to_string()));
        }
    }
    let complete = form.fill(ans);
    match tes.submit_evaluation(complete).await {
        Ok(_) => println!("[Info]::<TES>: Submit successfully"),
        Err(e) => eprintln!("[Error]::<TES>: Submit failed: {}", e),
    }
}

pub async fn fill(context: &Context) {
    login(context).await;
    println!(
        "[Info]::<TES>: ======================= Manual fill start ======================="
    );
    let tes = context.tes();

    let list = match tes.get_evaluation_list().await {
        Ok(list) => list,
        Err(e) => {
            eprintln!("[Error]::<TES>: Get list failed: {}", e);
            return;
        }
    };

    // 过滤出有用的部分
    let list: Vec<EvaluationListItem> = list.into_iter().filter(|item| !item.state).collect();

    for l in list {
        println!(
            "[Info]::<TES>: Course: {}, Teacher: {}",
            l.course, l.teacher
        );
        println!("[Info]::<TES>: Option is score, type the index");
        let form = match tes.get_evaluation_form(&l).await {
            Ok(f) => f,
            Err(e) => {
                eprintln!("[Error]::<TES>: Get form failed: {}", e);
                return;
            }
        };
        let mut ans: Vec<EvaluationAnswer> = Vec::with_capacity(form.questions.len());
        for (i, q) in form.questions.iter().enumerate() {
            println!("[Info]::<TES>: {}. {}", i + 1, q.name);
            if q.is_choice {
                let mut builder = tabled::builder::Builder::new();
                builder.push_record(["A", "B", "C", "D"]);
                builder.push_record([
                    &q.options[0].score.to_string(),
                    &q.options[1].score.to_string(),
                    &q.options[2].score.to_string(),
                    &q.options[3].score.to_string(),
                ]);
                crate::util::print_table(builder);
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
                ans.push(EvaluationAnswer::Choice(index));
            } else {
                ans.push(EvaluationAnswer::Completion(str.trim().to_string()));
            }
        }
        let complete = form.fill(ans);

        print!(
            "[Info]::<TES>: Finall score is {}. Press Enter to submit",
            complete.score()
        );
        std::io::stdout().flush().unwrap();
        let _ = std::io::stdin().read_line(&mut String::new()).unwrap();

        match tes.submit_evaluation(complete).await {
            Ok(_) => println!("[Info]::<TES>: Submit successfully"),
            Err(e) => eprintln!("[Error]::<TES>: Submit failed: {}", e),
        }
    }
    println!(
        "[Info]::<TES>: ======================== Manual fill end ========================"
    );
}

pub async fn auto(context: &Context) {
    print!("Warning!!! This function maybe not work as expected, and it will be fixed untill the next term. Press Enter to continue");
    std::io::stdout().flush().unwrap();
    let _ = std::io::stdin().read_line(&mut String::new()).unwrap();

    login(context).await;
    println!(
        "[Info]::<TES>: ======================= Auto fill start ======================="
    );
    let tes = context.tes();

    let list = match tes.get_evaluation_list().await {
        Ok(list) => list.into_iter().filter(|item| !item.state).collect::<Vec<EvaluationListItem>>(),
        Err(e) => {
            eprintln!("[Error]::<TES>: Get list failed: {}", e);
            return;
        }
    };

    for l in list {
        println!(
            "[Info]::<TES>: Course: {}, Teacher: {}",
            l.course, l.teacher
        );
        let form = match tes.get_evaluation_form(&l).await {
            Ok(f) => f,
            Err(e) => {
                eprintln!("[Error]::<TES>: Get form failed: {}", e);
                return;
            }
        };
        let mut ans: Vec<EvaluationAnswer> = Vec::with_capacity(form.questions.len());
        for (i, q) in form.questions.iter().enumerate() {
            if q.is_choice {
                if i == 0 {
                    ans.push(EvaluationAnswer::Choice(1));
                } else {
                    ans.push(EvaluationAnswer::Choice(0));
                }
            } else {
                ans.push(EvaluationAnswer::Completion("".to_string()));
            }
        }
        let complete = form.fill(ans);
        println!("[Info]::<TES>: Finall score is {}", complete.score());
        match tes.submit_evaluation(complete).await {
            Ok(_) => println!("[Info]::<TES>: Submit successfully"),
            Err(e) => eprintln!("[Error]::<TES>: Submit failed: {}", e),
        }
    }
    println!(
        "[Info]::<TES>: ======================== Auto fill end ========================"
    );
}
