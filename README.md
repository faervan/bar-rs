# bar-rs
<a href="https://github.com/iced-rs/iced">
  <img src="https://gist.githubusercontent.com/hecrj/ad7ecd38f6e47ff3688a38c79fd108f0/raw/74384875ecbad02ae2a926425e9bcafd0695bade/color.svg" width="130px">
</a>

A simple status bar, written using [iced-rs](https://github.com/iced-rs/iced/) (purely rust)

![image](https://github.com/user-attachments/assets/29daa606-3189-4355-bc04-a21e8f245f6f)

![2024-12-29_17-16](https://github.com/user-attachments/assets/199452ec-b5bc-4ac3-ac35-ef7aed732c2f)

Vertical status bars are only experimentally supported right now...
![2025-01-10_14-56](https://github.com/user-attachments/assets/f7bc78e5-56df-4c92-ba4d-f26180baea9b)

---

Currently bar-rs supports only a bare minimum of configuration. It works on Wayland compositors implementing the [wlr layer shell protocol](https://wayland.app/protocols/wlr-layer-shell-unstable-v1#compositor-support), but only features [hyprland](https://github.com/hyprwm/Hyprland/) modules for active workspace and window display.

For a list of all currently supported modules, see [the Wiki](https://github.com/Faervan/bar-rs/wiki#modules)

## ToC
- [Installation](#installation)
- [Extra dependencies](#extra-dependencies)
- [Usage](#usage)
- [Configuration](#configuration)
- [Hyprland configuration](#hyprland-configuration)
- [Logs](#logs)
- [Recommendations + feature requests](#recommendations--feature-requests)
- [Contributing](#contributing)
- [Extra credits](#extra-credits)

## Installation
To use bar-rs you have to build the project yourself (very straight forward on an up-to-date system like Arch, harder on "stable" ones like Debian due to outdated system libraries)

```sh
# Clone the project
git clone https://github.com/faervan/bar-rs.git
cd bar-rs

# Build the project - This might take a while
cargo build --release

# Install the bar-rs helper script to easily launch and kill bar-rs
bash install.sh

# Optional: Clean unneeded build files afterwards:
find target/release/* ! -name bar-rs ! -name . -type d,f -exec rm -r {} +
```

## Extra dependencies
bar-rs depends on the following cli utilities:
- free
- grep
- awk
- printf
- pactl
- wpctl
- playerctl

## Usage
Either launch bar-rs directly:

```sh
./target/release/bar-rs
# or using cargo:
cargo run --release
```

or using the `bar-rs` script (after installing it using the `install.sh` script)
```sh
bar-rs open
```

## Configuration
See [the Wiki](https://github.com/Faervan/bar-rs/wiki)

## Hyprland configuration
Add this line to your `~/.config/hypr/hyprland.conf` to launch bar-rs on startup:
```
exec-once = bar-rs open
```

To have the `hyprland.workspaces` module show some nice workspace icons, set rules for your workspaces like this:
```
workspace = 1, defaultName:󰈹
```
Find some nice icons to use [here](https://www.nerdfonts.com/cheat-sheet)

## Logs
If bar-rs is launched via the `bar-rs` script, it's logs are saved to `/tmp/bar-rs.log` and should only contain anything if there is an error.
If an error occurs and all dependencies are installed on your system, please feel free to open an [issue](https://github.com/faervan/bar-rs/issues)

## Recommendations + feature requests
If you have an idea on what could improve bar-rs, or you would like to see a specific feature implemented, please open an [issue](https://github.com/faervan/bar-rs/issues).

## Contributing
If you want to contribute, create an [issue](https://github.com/faervan/bar-rs/issues) about the feature you'd like to implement or comment on an existing one. You may also contact me on [discord](https://discord.com/users/738658712620630076).

## Extra credits
Next to all the great crates this projects depends on (see `Cargo.toml`) and the cli utils listed in [Extra dependencies](#extra-dependencies), bar-rs also uses [NerdFont](https://www.nerdfonts.com/) (see `assets/3270`)
