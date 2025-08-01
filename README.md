# bar-rs
<a href="https://github.com/iced-rs/iced">
  <img src="https://gist.githubusercontent.com/hecrj/ad7ecd38f6e47ff3688a38c79fd108f0/raw/74384875ecbad02ae2a926425e9bcafd0695bade/color.svg" width="130px">
</a>

A simple status bar, written using [iced-rs](https://github.com/iced-rs/iced/) (specifically the [pop-os fork](https://github.com/pop-os/iced/) of iced, which supports the [wlr layer shell protocol](https://wayland.app/protocols/wlr-layer-shell-unstable-v1))

> [!Note]
> `bar-rs` is currently undergoing a full rewrite, see [#24](https://github.com/Faervan/bar-rs/issues/24) for the reasons.<br>
> ~~Progress is a bit slow since I am working on some other, smaller projects which I will complete first.~~ <br>
> Development is active in the [`crabbar_rework`](https://github.com/faervan/bar-rs/tree/crabbar_rework) branch again.

![image](https://github.com/user-attachments/assets/c62d8399-0f80-4c3b-8cb8-a325db13fc32)

![image](https://github.com/user-attachments/assets/d71b0fc2-a9fb-43e9-b358-9b1f2cb3d487)

![image](https://github.com/user-attachments/assets/d0073653-01ed-4084-9c33-0d161cd98ec7)



Currently bar-rs supports only a small amount of configuration. It works on Wayland compositors implementing the [wlr layer shell protocol](https://wayland.app/protocols/wlr-layer-shell-unstable-v1#compositor-support), but right now only features [Hyprland](https://github.com/hyprwm/Hyprland/), [Niri](https://github.com/YaLTeR/niri/) and [Wayfire](https://github.com/WayfireWM/wayfire/) modules for active workspace and window display.

For a list of all currently supported modules, see [the Wiki](https://github.com/Faervan/bar-rs/wiki#modules)

## Features
- [x] Dynamic module activation/ordering
- [x] Hot config reloading
- [x] very basic style customization
- [x] basic vertical bar support
- [x] a base set of useful modules
- [x] Module interactivity (popups, buttons)
- [x] hyprland workspace + window modules
- [x] wayfire workspace + window modules
- [x] niri workspace + window modules
- [ ] sway workspace + window modules
- [ ] custom modules
- [ ] additional modules (wifi, pacman updates...)
- [ ] system tray support
- [ ] plugin api (for custom rust modules)
- [ ] custom fonts
- [ ] X11 support
- ...

## Installation
I aim for a release on the `AUR` after the [first milestone](https://github.com/Faervan/bar-rs/milestone/1) is reached. For now, you have to build bar-rs yourself.

<details>
<summary><h2>Building</h2></summary>
  
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
</details>

<details>
<summary><h2>Updating</h2></summary>

Enter the project directory again.

```sh
# Update the project
git pull

# Build the project - This will be considerably faster if you didn't clean the build files after installing
cargo build --release

# Optional: Clean unneeded build files afterwards:
find target/release/* ! -name bar-rs ! -name . -type d,f -exec rm -r {} +
```
</details>

<details>
<summary><h2>Extra dependencies</h2></summary>
  
bar-rs depends on the following cli utilities:
- free
- grep
- awk
- printf
- pactl
- wpctl
- playerctl
</details>

<details>
<summary><h2>Usage</h2></summary>
  
Launch bar-rs using the `bar-rs` script (after installing it using the `install.sh` script):
```sh
bar-rs open
```

Alternatively, you may launch bar-rs directly:

```sh
./target/release/bar-rs
# or using cargo:
cargo run --release
```
</details>

## Configuration
Example configurations can be found in [default_config](https://github.com/Faervan/bar-rs/tree/main/default_config).<br>
See [the Wiki](https://github.com/Faervan/bar-rs/wiki) for more.

## Logs
If bar-rs is launched via the `bar-rs` script, it's logs are saved to `/tmp/bar-rs.log` and should only contain anything if there is an error.
If an error occurs and all dependencies are installed on your system, please feel free to open an [issue](https://github.com/faervan/bar-rs/issues)

## Recommendations + feature requests
If you have an idea on what could improve bar-rs, or you would like to see a specific feature implemented, please open an [issue](https://github.com/faervan/bar-rs/issues).

## Contributing
If you want to contribute, create an [issue](https://github.com/faervan/bar-rs/issues) about the feature you'd like to implement or comment on an existing one. You may also contact me on [matrix](https://matrix.to/#/@faervan:matrix.org) or [discord](https://discord.com/users/738658712620630076).

Contributing by creating new modules should be pretty easy and straight forward if you know a bit about rust. You just have to implement the `Module` and `Builder` traits for your new module and register it in `src/modules/mod.rs`.<br>
Take a look at [docs.iced.rs](https://docs.iced.rs/iced/) for info about what to place in the `view()` method of the `Module` trait.

## Extra credits
Next to all the great crates this projects depends on (see `Cargo.toml`) and the cli utils listed in [Extra dependencies](#extra-dependencies), bar-rs also uses [NerdFont](https://www.nerdfonts.com/) (see `assets/3270`)
