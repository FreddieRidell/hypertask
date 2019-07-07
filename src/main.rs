extern crate hypertask;

use chrono::prelude::*;
use std::env;

fn get_now() -> DateTime<Utc> {
    Utc::now()
}

fn main() {
    let args: Vec<_> = env::args().collect();

    hypertask::run_cli(&get_now, &args).unwrap();
}
