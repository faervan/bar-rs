# Net
Names: `net.upload`, `net.download`

Shows the current network transfer speed. `net.upload` shows the upload speed, `net.download` shows the download speed.<br>
Both modules are updated once per second by a shared listener which reads stats from `/proc/net/dev` for the interface used by the default route (found in `/proc/net/route`), see [kernel.org](https://docs.kernel.org/filesystems/proc.html#network-device-information).

You can override the default settings defined in [Module Styling](./Modules.md) by setting them in this section: `module:net.upload` or `module:net.download`.
| Option | Description | Data type | Default |
| ------ | ----------- | --------- | ------- |
| icon | the icon to use | String | 󰕒 for `net.upload`, 󰇚 for `net.download` |

The speed is displayed in `B/s`, `KB/s` or `MB/s` depending on its magnitude.