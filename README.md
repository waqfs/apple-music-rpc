### apple-music-rpc
A small Rust daemon that receives playback notifications to display your current track in a Discord Rich Presence from Apple Music. There is no subprocesses, Apple Script calls, `osascript`, JavaScript runtime, or anything else.

No music artwork is uploaded to services for RPC, and artwork is only displayed if the iTunes search result is definitively accurate. This daemon does not repeatedly request song information, it receives notifications from macOS itself.

### Configuration
The activity, and other daemon properties, can be customized in the `config.toml` that is written into `~/Library/Application Support/apple-music-rpc/`. You can find the default config in [config.example.toml](config.example.toml).

The configuration contains the following properties:
```toml
[discord]
client_id = "1239490006054207550" # Application ID (default is from Vencord's discontinued RPC plugin)
ping_interval_seconds = 15 # interval between pings to Discord
reconnect_interval_seconds = 5 # interval between reconnection attempts

[activity]
details_format = "{title}" # first line of text, also rendered in the members sidebar
state_format = "{artist}" # second line of text
large_text_format = "{album}" # third line of text and also the hover text for the large image

[apple_music]
artwork_resolution = 512 # artwork width & height for Discord
timeout_seconds = 5 # maximum time to wait for an iTunes API response
query_results_limit = 20 # maximum number of results to search for the playing song
```

Currently supported templates include:
- `{title}`: track title
- `{artist}`: track artist
- `{album}`: track album/collection, empty string if not available
- `{genre}`: track genre, empty string if not available
- `{year}`: track release year, empty string if not available
- `{duration}`: track duration in `mm:ss` format
- `{duration_s}`: track duration in seconds
- `{play_count}`: track play count, empty string if not available
- `{artwork}`: track artwork url, empty string if not available

### Usage
While this is still in development, compile from source:
```zsh
cargo build --release && ./target/release/apple-music-rpc
```

You can test the ScriptingBridge and artwork resolution with the `--debug` flag:
```zsh
cargo build --release && ./target/release/apple-music-rpc --debug
```

### License
This project is licensed under GNU GPLv3, so you can use this for whatever, and forks must disclose source and be under the same license.
