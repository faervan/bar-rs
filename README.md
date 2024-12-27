# bar-rs
A simple status bar, written in [iced-rs](https://github.com/iced-rs/iced/) (purely rust)

Not (yet?) configurable and currently only working on [hyprland](https://github.com/hyprwm/Hyprland/).

Only contains the active hyprland workspaces and title of the focused window (left), the date and time (center) as well as the currently playing media as reported by playerctl, the sound volume, the battery capacity, cpu and memory usage (right).

## Installation
To use bar-rs you have to build the project yourself (very straight forward on a up-to-date system like Arch, harder on "stable" ones like Debian due to outdated system libraries)

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

## Hyprland configuration
[iced-rs](https://github.com/iced-rs/iced/) uses [winit](https://github.com/rust-windowing/winit/) as it's windowing shell, which has no support for the [`wlr layer shell protocol`](https://wayland.app/protocols/wlr-layer-shell-unstable-v1) yet, though there is [effort](https://github.com/rust-windowing/winit/pull/4044) made to implement it
For this reason, some hyprland rules are needed to make bar-rs behave as it should:
```
windowrulev2 = pin, class:(bar-rs)
windowrulev2 = float, class:(bar-rs)
windowrulev2 = nofocus, class:(bar-rs)
windowrulev2 = noborder, class:(bar-rs)
windowrulev2 = move 0 0, class:(bar-rs)
windowrulev2 = decorate 0, class:(bar-rs)
windowrulev2 = rounding 0, class:(bar-rs)
windowrulev2 = opacity 0, onworkspace:f[0], class:(bar-rs)
windowrulev2 = noblur 1, onworkspace:f[0], class:(bar-rs)
```

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

## Logs
are saved to `/tmp/bar-rs.log` and should only contain anything if there is an error.
If an error occurs and all dependencies are installed on your system, please feel free to open an [issue](https://github.com/faervan/bar-rs/issues)

## Extra credits
Next to all the great crates this projects depends on (see `Cargo.toml`) and the cli utils listed in [Extra dependencies](#extra-dependencies), bar-rs also uses [NerdFont](https://www.nerdfonts.com/) (see `assets/3270`)
