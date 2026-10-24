# Widgets and shortcuts

Android provides four shortcuts: **Add transaction**, **Transaction history**,
**Chat with agent**, and **Add data**. They open the same authenticated screens
as normal navigation. A pending shortcut survives sign-in and a required password
reset. Pressing Back returns to the dashboard. Opening a shortcut while Khata is
already running reuses the activity.

## Home screen

Long-press an empty area of the home screen, open **Widgets**, and find **Khata**:

- **Khata quick actions** contains all four shortcuts and can be resized.
- **Khata single shortcut** lets you choose one action when adding the widget.
  Launchers that support widget reconfiguration can change that selection later.

Long-press the Khata app icon for the same four launcher shortcuts. Inside the
app, **More → Widgets & shortcuts** explains both options and offers a button
to pin the quick-actions widget on supported launchers.

## Quick Settings

Open **More → Widgets & shortcuts**, then select **Add tile** for each desired
action. Android 13 and later show the system's tile-placement confirmation. On
older Android versions, swipe down twice, choose **Edit**, and drag the desired
Khata tiles into the panel. Each tile can be added or removed independently.

Tiles request device unlock before opening financial screens on a locked device.
Shortcuts carry only an allowlisted destination: they never create a transaction,
send a chat message, import data, or display financial details outside the app.
Widgets have no periodic network or background update work.

## Web

An installed Khata web app exposes the same four shortcuts on browsers that
support manifest shortcuts. Manual entry is available at `/add`; the remaining
destinations are `/transactions`, `/chat` and `/upload`. Signing in or completing
a required password reset preserves the chosen destination. Browser support for
installed-app shortcut menus varies.

## Validation

Navigation regression tests cover destination allowlisting, cold starts, setup,
sign-in, forced password reset, warm launches and logout. UI checks use synthetic
local data, including failed and successful manual saves. Android widgets and
tiles use explicit immutable activity PendingIntents, with the API 34 tile
launch overload and a guarded fallback for Android 8–13.


Verified release: `v0.45.16`, GitHub Actions run `40068680696`, versionCode 202,
signed by the same certificate as `v0.45.15`. All 51 release unit tests and Android
release lint passed. Native checks covered cold/warm navigation, draft retention
on failed saves, successful entry, rotation, widget placement/configuration and
widget actions, including retaining a single widget selection across an update.
Seven browser checks cover all four authenticated destinations, save failure and
success, and required password reset. The matching CI web artifact is deployed.
Physical-phone and manufacturer-specific launcher behavior remain unverified.
