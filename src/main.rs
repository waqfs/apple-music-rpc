use std::{env, path::PathBuf};

use crate::{artwork::ArtworkResolver, config::Config, music::AppleMusicBridge};

mod artwork;
mod config;
mod discord;
mod music;

fn main() {
    daemon().unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });
}

fn daemon() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    let config_path = get_custom_config_path(&args)?.unwrap_or(Config::default_path()?);
    let config = Config::get(&config_path)?;

    if args.iter().any(|str| str == "--debug") {
        debug(config)?;
        return Ok(());
    }

    let discord = discord::spawn_discord_thread(config.discord.clone());
    Ok(())
}

fn debug(config: Config) -> Result<(), String> {
    let music = AppleMusicBridge::new()?;
    let state = music.get_state()?;
    println!("Current state: {:?}", state);

    let artwork_resolver = ArtworkResolver::new(&config);
    let url = match state.track {
        Some(track) => artwork_resolver.resolve(&track)?,
        _ => Err("error: failed to get track".to_string())?,
    };
    println!("Artwork URL: {}", url);

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
