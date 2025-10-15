use tabled::{
    builder::Builder,
    settings::{Alignment, Style},
};

use time::{OffsetDateTime, PrimitiveDateTime, UtcOffset};

use std::collections::hash_map::RandomState;
use std::env;
use std::error::Error;
use std::hash::{BuildHasher, Hasher};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn simple_once_rand() -> u64 {
    let state = RandomState::new();
    let mut hasher = state.build_hasher();
    // 结合时间和内存随机性
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    hasher.write_u128(now);
    hasher.write_usize(&now as *const _ as usize);
    hasher.finish()
}

pub fn simple_rand_range(min: u64, max: u64) -> u64 {
    let range = max - min + 1;
    let product = (simple_once_rand() as u128) * (range as u128);
    min + (product >> 64) as u64
}

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
