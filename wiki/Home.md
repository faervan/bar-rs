# Welcome to the bar-rs wiki!

While the configuration options aren't extensive at the moment, it's still good to know what tools you've got!<br>
There are some configuration examples at [default_config](https://github.com/Faervan/bar-rs/blob/main/default_config)

*If you find that this wiki contains wrong information or is missing something critical, please open an [issue](https://github.com/Faervan/bar-rs/issues/new?template=Blank+issue).*

## Config path
On Linux, the config path is `$XDG_DATA_HOME/bar-rs/bar-rs.ini` or `$HOME/.local/share/bar-rs/bar-rs.ini`

**Example:**<br>
`/home/alice/.config/bar-rs/bar-rs.ini`

If it isn't, you may check [here](https://docs.rs/directories/latest/directories/struct.ProjectDirs.html#method.config_local_dir)

## Syntax
bar-rs uses an ini-like configuration (as provided by [configparser](https://docs.rs/configparser/latest/configparser/)), which should be pretty easy to understand and use.

It looks like this:
```ini
[section]
key = value
```

## Data types
| Data type | Description | Examples |
| --------- | ----------- | -------- |
| bool | Either yes or no | `true` or `false`, `1` or `0`, `enabled` or `disabled`... |
| Color | A color as defined in the [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/) | `rgba(255, 0, 0, 0.5)`, `blue`, `rgb(255, 255, 255)` |
| String | Just a String | `DP-1` |
| float | A floating point number | `20`, `5.8` |
| u32 | A positive integer of range $2^{32}$ (0-4294967295) | `0`, `50`, `1920` |
| usize | A positive integer of range 0 - a lot (depends on your architecture, but probably enough) | `0`, `100000` |
| Value list | A list of values, separated by spaces. | `20 5 20` | 
| Insets | A list of four values, representing all four directions (usually top, right, bottom and right). If one value is provided, it is used for all four sides. If two values are provided, the first is used for top and bottom and the second for left and right. | `0 20 5 10`, `0`, `0 10` |

## General
The general section contains three options:
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| monitor | The monitor on which bar-rs should open. If this is set, bar-rs will override the default values of `width` and `height` (only the defaults, not the ones you specify). | String | / |
| hot_reloading | Whether bar-rs should monitor the config file for changes | bool | true |
| hard_reloading | Whether bar-rs should reopen and reload all modules (required for `anchor`, `width`, `height`, `margin` and e.g. workspace names set in the `niri.workspaces` module to be hot-reloadable) | bool | false |
| anchor | The anchor to use. Can be `top`, `bottom`, `left` or `right`. This decides whether the bar is vertical or not. | String | top |
| kb_focus | Defines whether bar-rs should be focusable. Can be `none` (no focus), `on_demand` (when you click on it) or `exclusive` (always stay focused). | String | none |

**Example:**
```ini
[general]
monitor = DP-1
hot_reloading = true
hard_reloading = false
anchor = top
```

## General Styling
Some of these options might get overwritten by module-specific settings.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| background | Background color of the status bar | Color | rgba(0, 0, 0, 0.5) |
| width | The total width of the bar. The default depends on whether the bar is vertical or horizontal. | u32 | 30 or 1920 |
| height | The total height of the bar. The default depends on whether the bar is vertical or horizontal. | u32 | 1080 or 30 |
| margin | The margin between the bar and the screen edge, depending on the anchor. | float | 0 |
| padding | The padding between the bar edges and the actual contents of the bar. | Insets (float) | 0 |
| spacing | Space between the modules, can be different for left, center and right | Value list (float) | 20 10 15 |

**Example:**
```ini
[style]
background = rgba(0, 0, 0, 0.5)
width = 1890
height = 30
margin = 5
padding = 0
spacing = 20 5 20
```

## Module Styling
This section sets default values for all modules, which can be overridden for each module individually (see below).
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

## Modules
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
| cpu | Shows the current CPU usage |
| memory | Shows the current memory usage |
| time | Shows the local time |
| date | Shows the local date |
| battery | Shows the current capacity and remaining time |
| media | Shows the currently playing media as reported by `playerctl` |
| volume | Shows the current audio volume as reported by `wpctl`, updated by `pactl` |
| hyprland.window | Shows the title of the currently focused window |
| hyprland.workspaces | Shows the currently open workspaces |
| wayfire.window | Shows the title of the currently focused window |
| wayfire.workspaces | Shows the currently open workspace |
| niri.window | Shows the title or app_id of the currently focused window |
| niri.workspaces | Shows the currently open workspaces |

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

### Resolvers
Resolvers are can be used instead of module names and are mapped to modules on specific conditions.

Currently bar-rs has two resolvers: **window** and **workspaces**, which map to `hyprland.window`, `wayfire.window` or `niri.window` or `hyprland.workspaces`, `wayfire.workspaces` or `niri.workspaces`, respectively, depending on the environment variable `XDG_CURRENT_DESKTOP`.

Defined in [src/resolvers.rs](https://github.com/Faervan/bar-rs/blob/main/src/resolvers.rs)

### Cpu
Name: `cpu`

You can override the default settings defined in [Module Styling](#module-styling) by setting them in this section.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| icon | the icon to use | String | 󰻠 |

### Memory
Name: `memory`

You can override the default settings defined in [Module Styling](#module-styling) by setting them in this section.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| icon | the icon to use | String | 󰍛 |

### Battery
Name: `battery`

You can override the default settings defined in [Module Styling](#module-styling) by setting them in this section.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| format | The format of this module | String | {{capacity}}% ({{hours}}h {{minutes}}min left) |

### Volume
Name: `volume`

You can override the default settings defined in [Module Styling](#module-styling) by setting them in this section.

### Media
Name: `media`

You can override the default settings defined in [Module Styling](#module-styling) by setting them in this section.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| icon | the icon to use | String |  |
| max_length | the maximum character length to show | usize | 35 |
| max_title_length | the maximum character length of the title part of the media. Only applies if `max_length` is reached and the media has an artist | usize | 20 |

### Date
Name: `date`

You can override the default settings defined in [Module Styling](#module-styling) by setting them in this section.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| icon | the icon to use | String |  |
| format | How to format the date. See [chrono](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) for the syntax. | String | `%a, %d. %b` |

### Time
Name: `time`

You can override the default settings defined in [Module Styling](#module-styling) by setting them in this section.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| icon | the icon to use | String |  |
| format | How to format the time. See [chrono](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) for the syntax. | String | `%H:%M` |

### Hyprland window
Name: `hyprland.window`

You can override the default settings defined in [Module Styling](#module-styling) by setting them in this section.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| max_length | the maximum character length of the title | usize | 25 |

### Hyprland workspaces
Name: `hyprland.workspaces`

You can override the default settings defined in [Module Styling](#module-styling) by setting them in this section.
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

To change the workspace icons, see [here](https://github.com/Faervan/bar-rs/wiki/WM-specific#hyprland).

### Wayfire window
Name: `wayfire.window`

You can override the default settings defined in [Module Styling](#module-styling) by setting them in this section.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| max_length | the maximum character length of the title | usize | 25 |

### Wayfire workspaces
Name: `wayfire.workspaces`

You can override the default settings defined in [Module Styling](#module-styling) by setting them in this section.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| icon_padding | Padding for the icon, useful to adjust the icon position. | Insets (float) | 0 |
| fallback_icon | Default icon to use | String | / |
| (row, column) | the name of the workspace | String | fallback_icon or `row/column` |

> \[!TIP]
> Find some nice icons to use as workspace names [here](https://www.nerdfonts.com/cheat-sheet)

**Example:**
```ini
[module:wayfire.workspaces]
fallback_icon = 
(0, 0) = 󰈹
(1, 0) = 
(2, 0) = 󰓓
(0, 1) = 
(1, 1) = 
```

### Niri window
Name: `niri.window`

You can override the default settings defined in [Module Styling](#module-styling) by setting them in this section.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| max_length | the maximum character length of the title | usize | 25 |
| show_app_id | Show the app_id instead of the window title | bool | false |

### Niri workspaces
Name: `niri.workspaces`

You can override the default settings defined in [Module Styling](#module-styling) by setting them in this section.
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
| Output: n | the name of the nth workspace on the given output (monitor) | String | / |
| output_order | the order of the workspaces, depending on their output (monitor) | Value list (String) | / |
| fallback_icon | the icon to use for unnamed workspaces | String |  |
| active_fallback_icon | the icon to use for unnamed workspaces when active | String |  |

> \[!TIP]
> Find some nice icons to use as workspace names [here](https://www.nerdfonts.com/cheat-sheet)

**Example:**
```ini
[module:niri.workspaces]
spacing = 15
padding = 0 12 0 6
icon_margin = -2 0 0 0
icon_size = 25
active_size = 25
output_order = DP-1, HDMI-A-1
DP-1: 1 = 󰈹
DP-1: 2 = 
DP-1: 3 = 󰓓
DP-1: 4 = 
DP-1: 5 = 
```
