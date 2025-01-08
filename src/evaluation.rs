use buaa_api::Context;
use buaa_api::exports::evaluation::EvaluationAnswer;

pub async fn list(context: &Context) {
    let evaluation = context.evaluation();
    evaluation.login().await.unwrap();

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
        let form = match evaluation.get_evaluation_form(&l).await {
            Ok(f) => f,
            Err(e) => {
                eprintln!("[Error]::<Evaluation>: Get form failed: {}", e);
                return;
            }
        };
        let mut ans: Vec<EvaluationAnswer> = Vec::with_capacity(form.questions.len());
        for q in form.questions {
            println!("[Info]::<Evaluation>: {}", q.name);
            if q.is_choice {
                let mut i = 1;
                for a in q.options {
                    println!("[Info]::<Evaluation>: {}. {}", i, a.score);
                    i += 1;
                }
                let mut input = String::new();
                loop {
                    input.clear();
                    std::io::stdin().read_line(&mut input).unwrap();
                    let input = input.trim().parse::<usize>().unwrap();
                }
            } else {
                let mut input = String::new();
                loop {
                    input.clear();
                    std::io::stdin().read_line(&mut input).unwrap();
                    let input = input.trim();
                    ans.push(EvaluationAnswer::Completion(input.to_string()));
                }
            }
        }
    }
}
