# Date and time modules

## Date
Name: `date`

Shows the date. Clicking the module opens a calendar popup with buttons to switch between months and years.

You can override the default settings defined in [Module Styling](./Modules.md) by setting them in this section: `module:date`.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| icon | the icon to use | String |  |
| format | How to format the date. See [chrono](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) for the syntax. | String | `%a, %d. %b` |

## Popup configuration
You can override the default settings defined in [Popup Styling](./Popups.md) by setting them in this section: `module_popup:date`.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| first_weekday | The first day of the week in the calendar popup. `monday` or `sunday`. | String | `monday` |
| icon_prev_year | The icon to use for the "previous year" button. | String | `◀◀` |
| icon_prev_month | The icon to use for the "previous month" button. | String | `◀` |
| icon_next_month | The icon to use for the "next month" button. | String | `▶` |
| icon_next_year | The icon to use for the "next year" button. | String | `▶▶` |

## Time
Name: `time`

Shows the time.

You can override the default settings defined in [Module Styling](./Modules.md) by setting them in this section: `module:time`.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| icon | the icon to use | String |  |
| format | How to format the time. See [chrono](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) for the syntax. | String | `%H:%M` |
