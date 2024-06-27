# usbrawmap

Easy way to assign actions to keyboard scancodes ignored by the USB keyboard driver on Windows.

## Example use case

![Sun Microsystems Type-6 Unix USB keyboard](https://user-images.githubusercontent.com/70631622/206449644-a244a4e7-43b5-4d0c-b0a1-831414b2c028.png)

This is a *Sun Microsystems Type-6 Unix USB keyboard*. It looks like your regular old QWERTY 102-odd key keyboard, but it's got a whole 11 keys on the left that you may not recognize. These keys were used on Sun's workstation OSes (e.g. Solaris) and are still recognized by most Unix-family OSes nowadays (including Linux), though they aren't usually assigned to anything.

**Problem**: Windows's built-in USB HID keyboard driver ignores scancodes it doesn't recognize and doesn't pass them to userland applications. This means that these keys
are, for all intents and purposes, unuseable on Windows.

This is where **usbrawmap** comes in: it uses USBPcap to tap into the raw USB packet stream and detect key presses on those special keys, and performs customizable
actions at each key event.

## Usage

0. Download the latest release and extract the .zip somewhere (for example in C:\usbrawmap).
1. Install **USBPcap** from its [official website](https://desowin.org/usbpcap/). You may need to reboot your computer.
2. Run `C:\Program Files\USBPcap\USBPcamCMD.exe` as administrator, you'll see something like this:
  ![image](https://user-images.githubusercontent.com/4533568/171448708-1c444841-91f5-420b-a848-1bf0fcec6208.png)
3. Search for your keyboard in the list. Here, 3 root hubs are displayed, and only the third one really contains devices, so the right hub is number 3.
4. Change the `general.driver` setting in `usbrawmap.toml` accordingly, for example by default the file contains:

    ```toml
    # Configuration file for Sun Type-6 and Type-7 keyboard

    [general]
    driver = 3 # use file \\.\USBPcap3

    ......
    ```

    If the hub containing your keyboard was number 1, you'd edit the file so it contains:

    ```toml
    # Configuration file for Sun Type-6 and Type-7 keyboard

    [general]
    driver = 1

    ......
    ```

5. Launch usbrawmap as administrator (right click, Run As Administrator) in the folder containing the configuration file. You should see this:
   ![image](https://user-images.githubusercontent.com/4533568/171449258-b84c2b82-e51d-4d0d-a77e-fcb6c336962b.png)
   
6. If the program started correctly, the config file has been loaded. Ensure the keys work as expected. Close the program.

7. Open the Task Scheduler (Win+R, `taskschd.msc`), use the sidebar on the right to create a task. Give it a simple name, like "usbrawmap", check "Run with highest privileges". In the "Actions" tab, add an action. Set the "program/script" field to "powershell.exe" and the arguments to `-noexit -WindowStyle hidden -command ".\usbrawmap.exe"`. Set the "start in" field to the directory containing usbrawmap.exe (the installation directory). In the "Conditions" tab, uncheck both checkboxes under "Power". In the "General" tab, make sure the "Run only when the user is logged on" box is checked. Save. 

8. Open the "Startup" directory (Win+R, `shell:startup`), right click, "New", "Shortcut". As the target, write `schtasks.exe /run /tn yourtaskname`, with the name of the task you created. Run the shortcut, check that the keys are working. Now, the program will start when the computer is powered on.

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

The key names (`CONTROL`, ...) must be in the VK format, with a full list available [here](src/vk.rs#L22) (another, maybe more complete list, is available [here](http://www.kbdedit.com/manual/low_level_vk_list.html)).

The provided configuration file contains mappings for the Type-6 keyboard shown above, but in practice any scancode can be mapped.

**Note:** to refresh the configuration file, kill the process (usbrawmap.exe) using the Task Manager, and start it again through the shortcut in `shell:startup`.

## License

This project is licenced under the MIT license.
