# Welcome to the bar-rs wiki!

While the configuration options aren't extensive at the moment, it's still good to know what tools you've got!<br>
There are some configuration examples at [default_config](https://github.com/Faervan/bar-rs/blob/main/default_config)

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
| Value list | A list of values, separated by commas | `20, 5, 20` | 

## General
The general section contains three options:
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| monitor | The monitor on which bar-rs should open. If this is set, bar-rs will override the default values of `width` and `height` (only the defaults, not the ones you specify). | String | / |
| hot_reloading | Whether bar-rs should monitor the config file for changes | bool | true |
| width | The total width of the bar. The default depends on whether the bar is vertical or horizontal. | u32 | 30 or 1920 |
| height | The total height of the bar. The default depends on whether the bar is vertical or horizontal. | u32 | 1080 or 30 |
| anchor | The anchor to use. Can be `top`, `bottom`, `left` or `right`. This decides whether the bar is vertical or not. | String | top |

**Example:**
```ini
[general]
monitor = DP-1
hot_reloading = true
anchor = top
```

## General Styling
Some of these options might get overwritten by module-specific settings.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| background | Background color of the status bar | Color | rgba(0, 0, 0, 0.5) |
| spacing | Space between the modules, can be different for left, center and right | Value list (float) | 20, 20, 20 |
| local_spacing | Space between the items in a module (e.g. icon and text) | float | 10 |
| font_size | Default font size | float | 16 |
| icon_size | Default icon size | float | 20 |
| text_color | Default text color | Color | white |
| icon_color | Default icon color | Color | white |

**Example:**
```ini
[style]
background = rgba(0, 0, 0, 0.5)
spacing = 20, 5, 20
font_size = 16
icon_size = 20
text_color = white
icon_color = white
local_spacing = 15
```

## Modules
The `[module]` section sets the enabled modules for each side:

**Example:**
```ini
[modules]
left = hyprland.workspaces, hyprland.window
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

### Cpu
Name: `cpu`
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| spacing | Space between the icon and text | float | As set by `[style] local_spacing` |
| font_size | font size | float | As set by `[style] font_size` |
| icon_size | icon size | float | As set by `[style] icon_size` |
| text_color | text color | Color | As set by `[style] text_color` |
| icon_color | icon color | Color | As set by `[style] icon_color` |
| icon | the icon to use | String | 󰻠 |

### Memory
Name: `memory`
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| spacing | Space between the icon and text | float | As set by `[style] local_spacing` |
| font_size | font size | float | As set by `[style] font_size` |
| icon_size | icon size | float | As set by `[style] icon_size` |
| text_color | text color | Color | As set by `[style] text_color` |
| icon_color | icon color | Color | As set by `[style] icon_color` |
| icon | the icon to use | String | 󰍛 |

### Battery
Name: `battery`
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| spacing | Space between the icon and text | float | As set by `[style] local_spacing` |
| font_size | font size | float | As set by `[style] font_size` |
| icon_size | icon size | float | As set by `[style] icon_size` |
| text_color | text color | Color | As set by `[style] text_color` |
| icon_color | icon color | Color | As set by `[style] icon_color` |

### Volume
Name: `volume`
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| spacing | Space between the icon and text | float | As set by `[style] local_spacing` |
| font_size | font size | float | As set by `[style] font_size` |
| icon_size | icon size | float | As set by `[style] icon_size` |
| text_color | text color | Color | As set by `[style] text_color` |
| icon_color | icon color | Color | As set by `[style] icon_color` |

### Media
Name: `media`
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| spacing | Space between the icon and text | float | As set by `[style] local_spacing` |
| font_size | font size | float | As set by `[style] font_size` |
| icon_size | icon size | float | As set by `[style] icon_size` |
| text_color | text color | Color | As set by `[style] text_color` |
| icon_color | icon color | Color | As set by `[style] icon_color` |
| icon | the icon to use | String |  |
| max_length | the maximum character length to show | usize | 35 |
| max_title_length | the maximum character length of the title part of the media. Only applies if `max_length` is reached and the media has an artist | usize | 20 |

### Date
Name: `date`
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| spacing | Space between the icon and text | float | As set by `[style] local_spacing` |
| font_size | font size | float | As set by `[style] font_size` |
| icon_size | icon size | float | As set by `[style] icon_size` |
| text_color | text color | Color | As set by `[style] text_color` |
| icon_color | icon color | Color | As set by `[style] icon_color` |
| icon | the icon to use | String |  |
| format | How to format the date. See [chrono](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) for the syntax. | String | `%a, %d. %b` |

### Time
Name: `time`
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| spacing | Space between the icon and text | float | As set by `[style] local_spacing` |
| font_size | font size | float | As set by `[style] font_size` |
| icon_size | icon size | float | As set by `[style] icon_size` |
| text_color | text color | Color | As set by `[style] text_color` |
| icon_color | icon color | Color | As set by `[style] icon_color` |
| icon | the icon to use | String |  |
| format | How to format the time. See [chrono](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) for the syntax. | String | `%H:%M` |

### Hyprland window
Name: `hyprland.window`
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| font_size | font size | float | As set by `[style] font_size` |
| text_color | text color | Color | As set by `[style] text_color` |
| max_length | the maximum character length of the title | usize | 25 |

### Hyprland workspaces
Name: `hyprland.workspaces`
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| spacing | Space between the icon and text | float | As set by `[style] local_spacing` |
| icon_size | icon size | float | As set by `[style] icon_size` |
| icon_color | icon color | Color | As set by `[style] icon_color` |
| active_color | the color for the currently focused workspace | Color | black |
| active_background | the background color for the currently focused workspace | Color | rgba(255, 255, 255, 0.5) |

To change the workspace icons, see [here](https://github.com/Faervan/bar-rs/wiki/WM-specific#hyprland).

### Wayfire window
Name: `wayfire.window`
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| font_size | font size | float | As set by `[style] font_size` |
| text_color | text color | Color | As set by `[style] text_color` |
| max_length | the maximum character length of the title | usize | 25 |

### Wayfire workspaces
Name: `wayfire.workspaces`
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| icon_size | icon size | float | As set by `[style] icon_size` |
| icon_color | icon color | Color | As set by `[style] icon_color` |
| (row, column) | the name of the workspace | String | `row/column` |

> \[!TIP]
> Find some nice icons to use as workspace names [here](https://www.nerdfonts.com/cheat-sheet)

**Example:**
```ini
[module:wayfire.workspaces]
(0, 0) = 󰈹
(1, 0) = 
(2, 0) = 󰓓
(0, 1) = 
(1, 1) = 
(2, 1) = 
(0, 2) = 
(1, 2) = 
(2, 2) = 
```

### Niri window
Name: `niri.window`
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| font_size | font size | float | As set by `[style] font_size` |
| text_color | text color | Color | As set by `[style] text_color` |
| max_length | the maximum character length of the title | usize | 25 |
| show_app_id | Show the app_id instead of the window title | bool | false |

### Niri workspaces
Name: `niri.workspaces`
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| spacing | Space between the icon and text | float | As set by `[style] local_spacing` |
| icon_size | icon size | float | As set by `[style] icon_size` |
| icon_color | icon color | Color | As set by `[style] icon_color` |
| active_color | the color for the currently focused workspace | Color | black |
| active_background | the background color for the currently focused workspace | Color | rgba(255, 255, 255, 0.5) |
| Output: n | the name of the nth workspace on the given output (monitor) | String | / |
| fallback_icon | the icon to use for unnamed workspaces | String | / |
| output_order | the order of the workspaces, depending on their output (monitor) | Value list (String) | / |

> \[!TIP]
> Find some nice icons to use as workspace names [here](https://www.nerdfonts.com/cheat-sheet)

**Example:**
```ini
[module:niri.workspaces]
spacing = 15
fallback_icon = 
output_order = DP-1, HDMI-A-1
DP-1: 1 = 󰈹
DP-1: 2 = 
DP-1: 3 = 󰓓
DP-1: 4 = 
DP-1: 5 = 
```
