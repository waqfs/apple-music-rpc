use std::{env, path::PathBuf};

use crate::config::Config;

mod config;

fn main() {
    main2().unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });
}

fn main2() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    let config_path = get_custom_config_path(&args)?.unwrap_or(Config::default_path()?);
    let config = Config::get(&config_path)?;
    Ok(())
}

fn get_custom_config_path(args: &[String]) -> Result<Option<PathBuf>, String> {
    let mut index = 0;
    let mut path: Option<PathBuf> = None;
    while index < args.len() {
        if args[index] == "--config" {
            let raw = args
                .get(index + 1)
                .ok_or_else(|| "error: --config flag requires a path".to_string())?;
            path = Some(PathBuf::from(raw));
            index += 2;
        } else {
            index += 1;
        }
    }
    Ok(path)
}
