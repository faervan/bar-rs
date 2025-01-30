# Cpu
Name: `cpu`

Shows the cpu usage, also has a popup which can show more stats, including each core individually.<br>
This module reads stats from `/proc/stat`, see [kernel.org](https://docs.kernel.org/filesystems/proc.html#miscellaneous-kernel-statistics-in-proc-stat)

You can override the default settings defined in [Module Styling](./Modules.md) by setting them in this section: `module:cpu`.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| icon | the icon to use | String | 󰻠 |

## Popup configuration
You can override the default settings defined in [Popup Styling](./Popups.md) by setting them in this section: `module_popup:cpu`.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| format | the format of the popup text | String | `Total: {{total}}%\nUser: {{user}}%\nSystem: {{system}}%\nGuest: {{guest}}%\n{{cores}}` |
| format_core | the format of the cpu core | String | `Core {{index}}: {{total}}%` |

both `format` and `format_core` support:
- `total`: The total cpu/core usage
- `user`: The userspace cpu/core usage
- `system`: the kernelspace cpu/core usage
- `guest`: the usage of processes running in a guest session

`format` additionally supports:
- `cores`: all cores ordered by their id (ascending), separated by line breaks

`format_core` additionally supports:
- `index`: The index of the core
