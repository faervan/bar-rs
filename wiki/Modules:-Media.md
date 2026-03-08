# Media
Name: `media`

Shows the currently playing media title and artist and offers basic playback control using a popup.<br>
This module depends on `playerctl`.

You can override the default settings defined in [Module Styling](./Modules.md) by setting them in this section: `module:media`.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| icon | the icon to use | String |  |
| max_length | The maximum character length to show | usize | 35 |
| max_title_length | The maximum character length of the title part of the media. Only applies if `max_length` is reached and the media has an artist | usize | 20 |
| players | The media players to track. Run `playerctl metadata --all-players` to find the names of the currently active players. | Value list (String) | spotify, kew |

## Popup configuration
You can override the default settings defined in [Popup Styling](./Popups.md) by setting them in this section: `module_popup:media`.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| cover_width | The width of the cover image. The height is set automatically such that it preserves the aspect ratio. | float | 260 |
| icon_previous | The icon to use for the "previous track" button. | String | 󰒮 |
| icon_play | The icon to use for the "play track" button. | String |  |
| icon_pause | The icon to use for the "pause track" button. | String |  |
| icon_next | The icon to use for the "next track" button. | String | 󰒭 |
| format | The format of the popup text | String | `{{title}}{{status}}\nin: {{album}}\nby: {{artist}}\n{{length}}` |
| format_length | The format of length of the media | String | `{{minutes}}min {{seconds}}sec` |

`format` supports:
- `title` (The title of the playing media)
- `artist` (The artist of the playing media)
- `album` (The album of the playing media)
- `status` (The status of the playing media: empty if playing, `" (paused)"` if paused)
- `length` (The length of the playing media, its format is determined by `format_length`)

`format_length` supports:
- `minutes`
- `seconds`
