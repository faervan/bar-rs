# Date and time modules
These modules are basically identical.

## Date
Name: `date`

Shows the date.

You can override the default settings defined in [Module Styling](./Modules.md) by setting them in this section.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| icon | the icon to use | String |  |
| format | How to format the date. See [chrono](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) for the syntax. | String | `%a, %d. %b` |

## Time
Name: `time`

Shows the time.

You can override the default settings defined in [Module Styling](./Modules.md) by setting them in this section.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| icon | the icon to use | String |  |
| format | How to format the time. See [chrono](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) for the syntax. | String | `%H:%M` |
