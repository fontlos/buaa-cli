use buaa_api::Context;

pub async fn exam(context: &Context) {
    let exam = context.app();
    match exam.get_exam().await {
        Ok(exams) => {
            let mut builder = tabled::builder::Builder::new();
            builder.push_record(["Name", "Start", "End", "Position"]);
            for e in exams.data {
                builder.push_record([
                    &e.name,
                    &e.start.to_string(),
                    &e.end.to_string(),
                    &e.position,
                ]);
            }
            crate::utils::print_table(builder);
        }
        Err(e) => eprintln!("[Error]::<Exam>: Failed to get exam list: {e}"),
    }
}
