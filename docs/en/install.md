English | [Deutsch](../de/install.md)

# Install Hows

Hows runs on Windows 10 and Windows 11.

## Download

There is no signed release yet. The Windows builds come from the **Release** workflow of this repository:

1. Open the **Actions** tab and select the **Release** workflow.
2. Open a successful run and download the `windows-installers` artifact under **Artifacts**. It is a ZIP file.
3. Extract the ZIP.

Once a version is published, its entry on the **Releases** page includes the same files.

Windows SmartScreen warns about the unsigned files on first start. [The SmartScreen warning](smartscreen.md) explains why and how to continue.

## Choose a file

The artifact holds the installer, the MSI, the portable ZIP, and `wix-UIExtension-wix3141rtm.zip`. You need one of the first three. If you pass the MSI on, keep the WiX archive with it. It is the source of the WiX dialogs and their license, not a way to install Hows.

| File | Best for |
|---|---|
| Installer ending in `-setup.exe` | Most people. Installs Hows for you and registers `.steps` files, so a double-click opens them in Hows. |
| Package ending in `.msi` | Deploying to many PCs with software distribution tools. Also registers `.steps` files. |
| `Hows_*_x64-portable.zip` | Portable use without installing. It contains `hows.exe`, `LICENSE`, and `THIRD-PARTY-NOTICES.md`. It does not register `.steps` files. |

## WebView2

Hows draws its window with Microsoft Edge WebView2. Windows 11 already includes it. If it is missing, both installers download it from Microsoft during setup. This download is the only network connection the installers make; after install, the running app does not open network connections.

`hows.exe` inside the portable ZIP does not install WebView2. On a PC without it, install the WebView2 runtime from Microsoft first.

## First start

- Hows opens its library and adds an icon to the notification area of the taskbar.
- The interface follows your Windows display language if Hows supports it (English, German, French, Spanish, Italian, Portuguese, or Dutch); otherwise, it uses English. You can change it under [Settings](settings.md#appearance).
- If another app already uses one of the recording shortcuts, Hows opens the settings and marks the shortcut. See [Keyboard shortcuts](hotkeys.md#change-a-shortcut).

## Update

Hows does not check for updates. After install, the running app does not open network connections. To update, download the new build and run its installer. Your guides and settings live outside the program folder, so they stay.

## Uninstall

Open Windows **Settings**, then **Apps**, then **Installed apps**, and uninstall **Hows**. For the portable ZIP, delete the unpacked folder.

Your guides and exports are not deleted; they stay in the folders you chose (by default in your Documents folder). Hows keeps its settings in a `settings.json` file in its folder under `%APPDATA%`; delete that folder if you want to remove them too.

## Build it yourself

You can also build Hows from source. The [project overview](../../README.md#build) lists the requirements and commands.

---

[Documentation overview](README.md) · Next: [The SmartScreen warning](smartscreen.md)
