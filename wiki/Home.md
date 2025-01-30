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
| u32 | A positive integer of range $2^{32}$ (0 to 4_294_967_295) | `0`, `50`, `1920` |
| i32 | A signed integer (positive or negative) of range $2^{32}$ (-2_147_483_648 to 2_147_483_647) | `-500`, `2147483647` |
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
