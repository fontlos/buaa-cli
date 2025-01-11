use buaa_api::{Context, Error};
use buaa_api::exports::evaluation::EvaluationAnswer;

use std::io::Write;

pub async fn login(context: &Context) {
    let evaluation = context.evaluation();
    // 尝试登录, 如果是登录过期, 就继续执行, 其他错误就直接返回
    match evaluation.login().await {
        Ok(()) => {
            println!("[Info]::<Evaluation>: Login successfully");
            return;
        }
        Err(Error::LoginExpired(_)) => println!("[Info]::<Evaluation>: Try refresh SSO token"),
        Err(e) => {
            eprintln!("[Error]::<Evaluation>: Login failed: {}", e);
            return;
        }
    }
    // 如果是登录过期就继续执行到这里, 尝试登录 SSO, 失败了就直接返回
    match context.login().await {
        Ok(_) => println!("[Info]::<Evaluation>: SSO refresh successfully"),
        Err(e) => {
            eprintln!("[Error]::<Evaluation>: SSO Login failed: {}", e);
            return;
        },
    }
    // SSO 登录成功, 尝试登录 Boya, 失败了就直接返回
    match evaluation.login().await {
        Ok(()) => println!("[Info]::<Boya>: Login successfully"),
        Err(e) => eprintln!("[Error]::<Boya>: Login failed: {}", e),
    }
}

pub async fn list(context: &Context) {
    login(context).await;

    let evaluation = context.evaluation();
    match evaluation.get_evaluation_list().await {
        Ok(list) => {
            let mut builder = tabled::builder::Builder::new();
            builder.push_record(["Course", "Teacher"]);
            for l in list {
                builder.push_record([&l.course, &l.teacher]);
            }
            crate::util::print_table(builder);
        }
        Err(e) => {
            eprintln!("[Error]::<Evaluation>: Get list failed: {}", e);
        }
    }
}

pub async fn fill(context: &Context) {
    login(context).await;
    println!("[Info]::<Evaluation>: ======================= Manual fill start =======================");
    let evaluation = context.evaluation();

    let list = match evaluation.get_evaluation_list().await {
        Ok(list) => list,
        Err(e) => {
            eprintln!("[Error]::<Evaluation>: Get list failed: {}", e);
            return;
        }
    };

    for l in list {
        println!("[Info]::<Evaluation>: Course: {}, Teacher: {}", l.course, l.teacher);
        println!("[Info]::<Evaluation>: Option is score, type the index");
        let form = match evaluation.get_evaluation_form(&l).await {
            Ok(f) => f,
            Err(e) => {
                eprintln!("[Error]::<Evaluation>: Get form failed: {}", e);
                return;
            }
        };
        let mut ans: Vec<EvaluationAnswer> = Vec::with_capacity(form.questions.len());
        for (i, q) in form.questions.iter().enumerate() {
            println!("[Info]::<Evaluation>: {}. {}", i+1, q.name);
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
            print!("[Info]::<Evaluation>: Type answer: ");
            std::io::stdout().flush().unwrap();
            let mut str = String::new();
            std::io::stdin().read_line(&mut str).unwrap();
            if q.is_choice {
                let index = match str.trim() {
                    "A"|"a" => 0,
                    "B"|"b" => 1,
                    "C"|"c" => 2,
                    "D"|"d" => 3,
                    _ => {
                        eprintln!("[Error]::<Evaluation>: Invalid choice");
                        return;
                    }
                };
                ans.push(EvaluationAnswer::Choice(index));
            } else {
                ans.push(EvaluationAnswer::Completion(str.trim().to_string()));
            }
        }
        let complete = form.fill(ans);
        match evaluation.submit_evaluation(complete).await {
            Ok(_) => println!("[Info]::<Evaluation>: Submit successfully"),
            Err(e) => eprintln!("[Error]::<Evaluation>: Submit failed: {}", e),
        }
    }
}

pub async fn auto(context: &Context) {
    login(context).await;
    println!("[Info]::<Evaluation>: ======================= Auto fill start =======================");
    let evaluation = context.evaluation();

    let list = match evaluation.get_evaluation_list().await {
        Ok(list) => list,
        Err(e) => {
            eprintln!("[Error]::<Evaluation>: Get list failed: {}", e);
            return;
        }
    };

    for l in list {
        println!("[Info]::<Evaluation>: Course: {}, Teacher: {}", l.course, l.teacher);
        let form = match evaluation.get_evaluation_form(&l).await {
            Ok(f) => f,
            Err(e) => {
                eprintln!("[Error]::<Evaluation>: Get form failed: {}", e);
                return;
            }
        };
        let mut ans: Vec<EvaluationAnswer> = Vec::with_capacity(form.questions.len());
        for (i, q) in form.questions.iter().enumerate() {
            println!("[Info]::<Evaluation>: {}. {}", i+1, q.name);
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
        match evaluation.submit_evaluation(complete).await {
            Ok(_) => println!("[Info]::<Evaluation>: Submit successfully"),
            Err(e) => eprintln!("[Error]::<Evaluation>: Submit failed: {}", e),
        }
    }
}
