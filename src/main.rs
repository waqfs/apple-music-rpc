use std::{env, path::PathBuf, println, sync::mpsc};

use crate::{
    config::{Config, DEFAULT_CONFIG},
    music::{artwork::ArtworkResolver, bridge::AppleMusicBridge},
};

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
    if args.iter().any(|str| str == "-h" || str == "--help") {
        print_usage();
        return Ok(());
    }
    if args.iter().any(|str| str == "--dump-default-config") {
        print!("{DEFAULT_CONFIG}");
        return Ok(());
    }

    let config_path = get_custom_config_path(&args)?.unwrap_or(Config::default_path()?);
    let config = Config::get(&config_path)?;

    if args.iter().any(|str| str == "--debug") {
        debug(config)?;
        return Ok(());
    }

    let discord = discord::spawn_discord_thread(config.discord.clone());

    let (song_update_tx, song_update_rx) = mpsc::channel::<()>();
    music::spawn_music_thread(config, song_update_rx, discord);
    song_update_tx.send(());
    music::notification::notify_on_song_update(song_update_tx);

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

fn print_usage() {
    println!("\napple-music-rpc v{}\n", env!("CARGO_PKG_VERSION"));
    println!("Usage: apple-music-rpc [--config <path>]");
    println!("Options:");
    println!("  --config <path>        Specify a custom config file path");
    println!(
        "  --debug                Test ScriptingBridge and artwork resolution for the current song"
    );
    println!("  --dump-default-config  Print the default config to stdout");
    println!("  -h, --help             Show this help message\n");
}
