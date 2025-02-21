# Disk usage
Name: `disk_usage`

Shows the disk usage, also has a popup which can show more stats.<br>
This module obtains the stats via the `statvfs` syscall, see [man7.org](https://man7.org/linux/man-pages/man3/statvfs.3.html)

You can override the default settings defined in [Module Styling](./Modules.md) by setting them in this section: `module:disk_usage`.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| icon | the icon to use | String | 󰦚 |
| path | some directory, which determines the filesystem of interest | String | `/` |
| format | the content of the module text | String | `{{used_perc}}%` |

## Popup configuration
You can override the default settings defined in [Popup Styling](./Popups.md) by setting them in this section: `module_popup:disk_usage`.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| format | the format of the popup text | String | `Total: {{total_gb}} GB\nUsed: {{used_gb}} GB ({{used_perc}}%)\nFree: {{free_gb}} GB ({{free_perc}}%)` |

`format` provides the following variables:
- `total`: The total filesystem space in mb
- `total_gb`: The total filesystem space in gb
- `used`: The used space in mb
- `used_gb`: The used space in gb
- `free`: The free space in mb
- `free_gb`: The free space in gb
- `used_perc`: the percentage of used space in the filesystem
- `free_perc`: the percentage of free space in the filesystem
