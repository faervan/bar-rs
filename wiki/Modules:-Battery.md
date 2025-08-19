# Battery
Name: `battery`

You can override the default settings defined in [Module Styling](./Modules.md) by setting them in this section: `module:battery`.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| format | The format of this module | String | `{{capacity}}%{{time_remaining}}` |
| format_time | The format of the remaining battery time left (to full or to empty) | String | ` ({{hours}}h {{minutes}}min left)` |

## Popup configuration
You can override the default settings defined in [Popup Styling](./Popups.md) by setting them in this section: `module_popup:battery`.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| format | The format of the popup text | String | `{{name}}: {{state}}\n\t{{icon}} {{capacity}}% ({{energy}} Wh)\n\thealth: {{health}}%{{time_remaining}}\n\tmodel: {{model}}` |
| format_time | The format of the remaining battery time left (to full or to empty) | String | `\n\t{{hours}}h {{minutes}}min remaining` |

`format` supports:
- `name` (The name of the battery)
- `state` (The charging state of the battery)
- `icon` (The icon of the battery)
- `capacity` (The capacity of the battery)
- `energy` (The energy of the battery, in `Wh`)
- `health` (The health of the battery: energy_full / energy_full_design)
- `time_remaining` (The remaining battery time left (to full or to empty))
    - Control this by setting a custom format for `format_time`
    - This will be empty when the remaining time cannot be calculated
- `model` (The battery model)

`format_time` supports:
- `hours`
- `minutes`
