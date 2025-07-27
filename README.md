# crabbar
<a href="https://github.com/iced-rs/iced">
  <img src="https://gist.githubusercontent.com/hecrj/ad7ecd38f6e47ff3688a38c79fd108f0/raw/74384875ecbad02ae2a926425e9bcafd0695bade/color.svg" width="130px">
</a>

## Rewrite
`bar-rs` is currently undergoing a full rewrite (happening in this branch) and is renamed to `crabbar`.
See [#24](https://github.com/Faervan/bar-rs/issues/24) for the motivation behind this and [#25](https://github.com/Faervan/bar-rs/pull/25) for the to-do list.

## Features
- [-] Dynamic module activation/ordering
- [-] Hot config reloading
- [-] very basic style customization
- [-] basic vertical bar support
- [-] a base set of useful modules
- [-] Module interactivity (popups, buttons)
- [-] hyprland workspace + window modules
- [-] wayfire workspace + window modules
- [-] niri workspace + window modules
- [-] sway workspace + window modules
- [-] custom modules
- [-] additional modules (wifi, pacman updates...)
- [-] system tray support
- [-] plugin api (for custom rust modules)
- [-] custom fonts
- [-] X11 support
- ...

## Recommendations + feature requests
If you have an idea on what could improve `crabbar`, or you would like to see a specific feature implemented, please open an [issue](https://github.com/faervan/crabbar/issues).

## Contributing
If you want to contribute, create an [issue](https://github.com/faervan/crabbar/issues) about the feature you'd like to implement or comment on an existing one. You may also contact me on [matrix](https://matrix.to/#/@faervan:matrix.org) or [discord](https://discord.com/users/738658712620630076).

Contributing by creating new modules should be pretty easy and straight forward if you know a bit about rust. You just have to implement the `Module` and `Builder` traits for your new module and register it in `src/modules/mod.rs`.<br>
Take a look at [docs.iced.rs](https://docs.iced.rs/iced/) for info about what to place in the `view()` method of the `Module` trait.

## Extra credits
Next to all the great crates this projects depends on (see `Cargo.toml`), `crabbar` also uses [NerdFont](https://www.nerdfonts.com/) (see `assets/3270`)
