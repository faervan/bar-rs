# Hyprland modules
Add this line to your `~/.config/hypr/hyprland.conf` to launch bar-rs on startup:
```
exec-once = bar-rs open
```

bar-rs supports two modules for the [Hyprland](https://github.com/hyprwm/Hyprland/) wayland compositor:

## Hyprland window
Name: `hyprland.window`

You can override the default settings defined in [Module Styling](./Modules.md) by setting them in this section: `module:hyprland.window`.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| max_length | the maximum character length of the title | usize | 25 |

## Hyprland workspaces
Name: `hyprland.workspaces`

You can override the default settings defined in [Module Styling](./Modules.md) by setting them in this section: `module:hyprland.workspaces`.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| icon_padding | Padding for the icon, only useful with a background or border. | Insets (float) | 0 |
| icon_background | Background of the icons. | Color | None |
| icon_border_color | Color of the border around the icons. | Color | / |
| icon_border_width | Width of the border around the icons. | float | 1 |
| icon_border_radius | Radius of the border around the icons. | Insets (float) | 0 |
| active_padding | Padding for the active icon, only useful with a background or border. | Insets (float) | 0 |
| active_size | Size of the currently active icon. | float | 20 |
| active_color | the color for the currently focused workspace | Color | black |
| active_background | the background color for the currently focused workspace | Color | rgba(255, 255, 255, 0.5) |
| active_border_color | Color of the border around the active icon. | Color | / |
| active_border_width | Width of the border around the active icon. | float | 1 |
| active_border_radius | Radius of the border around the active icon. | Insets (float) | 0 |

To have the `hyprland.workspaces` module show some nice workspace icons, set rules for your workspaces like this:
```
workspace = 1, defaultName:󰈹
```

> \[!TIP]
> Find some nice icons to use as workspace names [here](https://www.nerdfonts.com/cheat-sheet)
