English | [Deutsch](../de/smartscreen.md)

# The SmartScreen warning

Hows builds are not code-signed yet. Microsoft Defender SmartScreen warns about programs whose publisher it cannot verify, so the first time you start the installer or `hows.exe` from the portable ZIP, Windows may show **Windows protected your PC**.

## What the warning means

SmartScreen does not know who published the file. It does not mean that a virus was found. Still, only run files you trust:

- Download Hows only from the [Releases](https://github.com/swayaa/hows/releases) page of this repository. See [Install Hows](install.md#download).
- If you prefer not to trust a prebuilt file, [build Hows yourself](../../README.md#build).

## Start Hows anyway

1. In the blue SmartScreen window, click **More info**.
2. Check that the app name matches the file you downloaded.
3. Click **Run anyway**.

## If the file is blocked

Windows marks files downloaded from the internet. If a downloaded ZIP or its files are blocked:

1. Right-click the downloaded ZIP and choose **Properties**.
2. On the **General** tab, select **Unblock** and click **OK**.
3. Extract the ZIP again.

Some organizations manage SmartScreen centrally. If you see no **Run anyway** button, ask your IT department to allow Hows.

## Signing and reputation

Code signing is planned. Signed builds carry a verified publisher name. Whether SmartScreen still warns then depends on the reputation that publisher and the file have built up with SmartScreen, so a new signed release can still show the warning for a while.

---

[Documentation overview](README.md) · Next: [Record a guide](recording.md)
