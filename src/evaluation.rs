use buaa_api::Context;

pub async fn list(context: &Context) {
    let evaluation = context.evaluation();

    match evaluation.get_evaluation_list().await {
        Ok(list) => {
            println!("[Info]::<Evaluation>: ======================= List start =======================");
            for l in list {
                println!("[Info]::<Evaluation>: Course name: {} | Teacher name: {}", l.course, l.teacher);
            }
            println!("[Info]::<Evaluation>: ======================== List end ========================");
        }
        Err(e) => {
            eprintln!("[Error]::<Evaluation>: Get list failed: {}", e);
        }
    }
}
