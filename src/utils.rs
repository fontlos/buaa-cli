use tabled::{
    builder::Builder,
    settings::{Alignment, Style},
};

use time::{OffsetDateTime, PrimitiveDateTime, UtcOffset};

use std::env;
use std::error::Error;
use std::path::PathBuf;

pub fn get_datetime() -> PrimitiveDateTime {
    let now_utc = OffsetDateTime::now_utc();
    let local_offset = UtcOffset::from_hms(8, 0, 0).expect("Failed to create local offset");
    let now_local = now_utc.to_offset(local_offset);
    PrimitiveDateTime::new(now_local.date(), now_local.time())
}

pub fn get_path(path: &str) -> Result<PathBuf, Box<dyn Error>> {
    let file_path = env::current_exe()?
        .parent()
        .ok_or("Failed to get parent directory")?
        .to_path_buf()
        .join(path);
    Ok(file_path)
}

pub fn print_table(builder: Builder) {
    let table = builder
        .index()
        .build()
        .with(Style::modern_rounded())
        .with(Alignment::center())
        .with(Alignment::center_vertical())
        .to_string();
    println!("{}", table);
}
