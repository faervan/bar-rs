# Wayfire modules
Add this to your `~/.config/wayfire.ini` to launch bar-rs on startup:
```ini
[autostart]
bar = bar-rs open
```

bar-rs supports two modules for the [Wayfire](https://github.com/WayfireWM/wayfire/) wayland compositor:

## Wayfire window
Name: `wayfire.window`

Shows the name of the currently open window.

You can override the default settings defined in [Module Styling](./Modules.md) by setting them in this section: `module:wayfire.window`.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| max_length | the maximum character length of the title | usize | 25 |

## Wayfire workspaces
Name: `wayfire.workspaces`

Shows the name of the currently focused workspace.

You can override the default settings defined in [Module Styling](./Modules.md) by setting them in this section: `module:wayfire.workspaces`.
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
