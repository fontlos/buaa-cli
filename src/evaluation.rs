use buaa_api::Context;

pub async fn list(context: &Context) {
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
