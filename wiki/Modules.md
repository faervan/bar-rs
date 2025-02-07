# Modules
The `[module]` section sets the enabled modules for each side:

**Example:**
```ini
[modules]
left = workspaces, window
center = date, time
right = media, volume, cpu, memory
```

The following modules are currently available:

| Module | Description |
| ------ | ----------- |
| [cpu](./Modules:-CPU.md) | Shows the current CPU usage |
| [memory](./Modules:-Memory.md) | Shows the current memory usage |
| [time](./Modules:-Date-and-Time.md) | Shows the local time |
| [date](./Modules:-Date-and-Time.md) | Shows the local date |
| [battery](./Modules:-Battery.md) | Shows the current capacity and remaining time |
| [media](./Modules:-Media.md) | Shows the currently playing media as reported by `playerctl` |
| [volume](./Modules:-Volume.md) | Shows the current audio volume as reported by `wpctl`, updated by `pactl` |
| [hyprland.window](./Modules:-Hyprland.md) | Shows the title of the currently focused window |
| [hyprland.workspaces](./Modules:-Hyprland.md) | Shows the currently open workspaces |
| [wayfire.window](./Modules:-Wayfire.md) | Shows the title of the currently focused window |
| [wayfire.workspaces](./Modules:-Wayfire.md) | Shows the currently open workspace |
| [niri.window](./Modules:-Niri.md) | Shows the title or app_id of the currently focused window |
| [niri.workspaces](./Modules:-Niri.md) | Shows the currently open workspaces |

To configure modules individually use a section name like this:
```ini
[module:{{name}}]
```
where `{{name}}` is the name of the module, e. g. `cpu`

**Example:**
```ini
[module:time]
icon_size = 24
format = %H:%M

[module:hyprland.workspaces]
active_color = black
active_background = rgba(255, 255, 255, 0.5)
```

## Module Styling
section name: `[module_style]`
This section sets default values for all modules, which can be overridden for each module individually.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| background | Background color of the status bar | Color | None |
| spacing | Space between the modules, can be different for left, center and right | Value list (float) | 10 |
| margin | The margin around this module. | Insets (float) | 0 |
| padding | The padding surrounding the module content. | Insets (float) | 0 |
| font_size | Default font size | float | 16 |
| icon_size | Default icon size | float | 20 |
| text_color | Default text color | Color | white |
| icon_color | Default icon color | Color | white |
| text_margin | The margin around the text of this module (can be used adjust the text position, negative values allowed). | Insets (float) | 0 |
| icon_margin | The margin around the icon of this module (can be used adjust the icon position, negative values allowed). | Insets (float) | 0 |
| border_color | The color of the border around this module. | Color | None |
| border_width | The width of the border. | float | 1 |
| border_radius | The radius (corner rounding) of the border. | Insets (float) | 0 |
| on_click | A command to be executed when you click the module with the left mouse button. | String | / |
| on_middle_click | A command to be executed when you click the module with the middle mouse button. | String | / |
| on_right_click | A command to be executed when you click the module with the right mouse button. | String | / |

### Resolvers
Resolvers are can be used instead of module names and are mapped to modules on specific conditions.

Currently bar-rs has two resolvers: **window** and **workspaces**, which map to `hyprland.window`, `wayfire.window` or `niri.window` or `hyprland.workspaces`, `wayfire.workspaces` or `niri.workspaces`, respectively, depending on the environment variable `XDG_CURRENT_DESKTOP`.

Defined in [src/resolvers.rs](https://github.com/Faervan/bar-rs/blob/main/src/resolvers.rs)

