# WM (or compositor) specific configuration

## Hyprland
Add this line to your `~/.config/hypr/hyprland.conf` to launch bar-rs on startup:
```
exec-once = bar-rs open
```

To have the `hyprland.workspaces` module show some nice workspace icons, set rules for your workspaces like this:
```
workspace = 1, defaultName:󰈹
```

> \[!TIP]
> Find some nice icons to use as workspace names [here](https://www.nerdfonts.com/cheat-sheet)

## Wayfire
Add this to your `~/.config/wayfire.ini` to launch bar-rs on startup:
```ini
[autostart]
bar = bar-rs open
```

## Niri
Add this to your `~/.config/niri/config.kdl` to launch bar-rs on startup:
```kdl
spawn-at-startup "bar-rs" "open"
```
