# Niri modules
Add this to your `~/.config/niri/config.kdl` to launch bar-rs on startup:
```kdl
spawn-at-startup "bar-rs" "open"
```

bar-rs supports two modules for the [Niri](https://github.com/YaLTeR/niri) Wayland compositor:

## Niri window
Name: `niri.window`

This module shows the name or app_id of the currently focused window. A popup is also available.

You can override the default settings defined in [Module Styling](./Modules.md) by setting them in this section: `module:niri.window`.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| max_length | The maximum character length of the title | usize | 25 |
| show_app_id | Show the app_id instead of the window title | bool | false |

### Popup configuration
You can override the default settings defined in [Popup Styling](./Popups.md) by setting them in this section: `module_popup:niri.window`.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| format | The format of the popup text | String | `Title: {{title}}\nApplication ID: {{app_id}}\nWindow ID: {{window_id}}\nWorkspace ID: {{workspace_id}}` |

```ini
[module_popup:niri.window]
format = {{title}}\n{{app_id}}
```

This supports:
- `title` (The active window title)
- `app_id` (The active window's application id)
- `window_id` (The active window's id)
- `workspace_id` (The id of the active workspace)

## Niri workspaces
Name: `niri.workspaces`

This module shows the currently open workspaces and allows to change your workspace by clicking on a workspace icon.

You can override the default settings defined in [Module Styling](./Modules.md) by setting them in this section: `module:niri.workspaces`.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| icon_padding | Padding for the icon, only useful with a background or border. | Insets (float) | 0 |
| icon_background | Background of the icons. | Color | None |
| icon_border_color | Color of the border around the icons. | Color | / |
| icon_border_width | Width of the border around the icons. | float | 1 |
| icon_border_radius | Radius of the border around the icons. | Insets (float) | 0 |
| active_padding | Padding for the active icon, only useful with a background or border. | Insets (float) | 0 |
| active_size | Size of the currently active icon. | float | 20 |
| active_color | The color for the currently focused workspace | Color | black |
| active_background | The background color for the currently focused workspace | Color | rgba(255, 255, 255, 0.5) |
| active_border_color | Color of the border around the active icon. | Color | / |
| active_border_width | Width of the border around the active icon. | float | 1 |
| active_border_radius | Radius of the border around the active icon. | Insets (float) | 0 |
| Output: n | The name of the nth workspace on the given output (monitor) | String | / |
| output_order | the order of the workspaces, depending on their output (monitor) | Value list (String) | / |
| fallback_icon | The icon to use for unnamed workspaces | String |  |
| active_fallback_icon | The icon to use for unnamed workspaces when active | String |  |

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
