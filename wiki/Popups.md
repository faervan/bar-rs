# Popups
Extra windows that open on click to show some more info or allow for additional actions.

## Popup Styling
section name: `[popup_style]`
This section sets default values for all module popups, which can be overridden for each module individually.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| width | The width of the popup | i32 | 300 |
| height | The height of the popup | i32 | 300 |
| fill_content_to_size | Whether the content of the module should fill the entire width and height | bool | false |
| padding | The padding surrounding the popup content. | Insets (float) | 10 20 |
| text_color | Default text color | Color | white |
| icon_color | Default icon color | Color | white |
| font_size | Default font size | float | 14 |
| icon_size | Default icon size | float | 24 |
| text_margin | The margin around the text of this popup (can be used adjust the text position, negative values allowed). | Insets (float) | 0 |
| icon_margin | The margin around the icon of this popup (can be used adjust the icon position, negative values allowed). | Insets (float) | 0 |
| spacing | Space between elements in the popup | float | 0 |
| background | Background color of the popup | Color | rgba(255, 255, 255, 0.8) |
| border_color | The color of the border around this popup. | Color | None |
| border_width | The width of the border. | float | 0 |
| border_radius | The radius (corner rounding) of the border. | Insets (float) | 8 |
