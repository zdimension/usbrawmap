# usbrawmap
Easy way to assign actions to keyboard scancodes ignored by the USB keyboard driver on Windows.

## Example use case
![image](https://user-images.githubusercontent.com/4533568/171446111-476ad724-79f2-4a88-9beb-42dbd45f1818.png)

This is a *Sun Microsystems Type-6 Unix USB keyboard*. It looks like your regular old QWERTY 102-odd key keyboard, but it's got a whole 11 keys on the left that 
you may not recognize. These keys were used on Sun's workstation OSes (e.g. Solaris) and are still recognized by most Unix-family OSes
nowadays (including Linux), though they aren't usually assigned to anything.

**Problem**: Windows's built-in USB HID keyboard driver ignores scancodes it doesn't recognize and doesn't pass them to userland applications. This means that these keys
are, for all intents and purposes, unuseable on Windows.

This is where **usbrawmap** comes in: it uses USBPcap to tap into the raw USB packet stream and detect key presses on those special keys, and performs customizable
actions at each key event.

## Usage
1. Install **USBPcap** from its [official website](https://desowin.org/usbpcap/). You may need to reboot your computer.
2. Run `C:\Program Files\USBPcap\USBPcamCMD.exe` as administrator, you'll see something like this:
  ![image](https://user-images.githubusercontent.com/4533568/171448708-1c444841-91f5-420b-a848-1bf0fcec6208.png)
3. Search for your keyboard in the list. Here, 3 root hubs are displayed, and only the third one really contains devices, so the right hub is number 3.
4. Change the `general.driver` setting in `usbrawmap.toml` accordingly.
5. Launch usbrawmap as administrator in the folder containing the configuration file. You should see this:
   ![image](https://user-images.githubusercontent.com/4533568/171449258-b84c2b82-e51d-4d0d-a77e-fcb6c336962b.png)
   
## Customizing mappings
The configuration file (`usbrawmap.toml`) uses the following format:
```toml
[mappings.SCANCODE]
type = "keys"
keys = ["CONTROL", "O"]

[mappings.SCANCODE]
type = "program"
path = "notepad.exe"
```
`SCANCODE` is the USB scancode for the key, with a full list available [here](https://www.win.tue.nl/~aeb/linux/kbd/scancodes-14.html).

The key names (`CONTROL`, ...) must be in the VK format, with a full list available [here](src/vk.rs#L22).

The provided configuration file contains mappings for the Type-6 keyboard shown above, but in practice any scancode can be mapped.

## License
This project is licenced under the MIT license.
