# No Mind Wing on Steam

Set up on 2026-11-13. Nothing in the game talks to Steam yet; this is the partner-site side only.

## The app

- **App ID: 1309380.** Reused, not new: it is Henry's old, never-released app "Rock Collector", which Henry ruled abandoned. No Steam Direct fee was paid.
- It is still named Rock Collector on the partner site and its store page. Its old Windows build and depot (1309381, "Rock Collector Content") are still there.
- Still unreleased: package hidden, no price, release state "Unavailable". Nothing public was touched (store page, price, release date, visibility).
- `steam_appid.txt` beside this file holds `1309380`.

## What was changed

Only one setting, then one configuration publish.

| Setting | Where on partner.steamgames.com | Was | Now |
|---|---|---|---|
| Enable ISteamUGC for file transfer | App 1309380, Edit Steamworks Settings, Workshop, General (`/apps/workshop/1309380`), "ISteamUGC file transfer" | off | **on** |

Pressing that section's Save also wrote the Workshop form's other options exactly as they stood: Ready-To-Use Content on, Required DLC on, the rest off. They appeared as a new "policies" section in the diff.

Then the change was published from the Publish tab (`/apps/publishing/1309380`): Prepare for Publishing, Publish to Steam, Really Publish. Nothing else was pending. The site answered "Publishing successful!" and "Since your app isn't released, these changes are only accessible for app owners with a release state override package." It configures; it does not release.

ISteamUGC is what the game will use to upload, search and download ship designs on its own screens, so the public design library never needs the Workshop's web pages.

## What was checked and left alone

- **Workshop visibility** (same page, right column): "Only Developers". Only accounts on the Steamworks team can see items. Fine for development. Players and outside testers need it changed later. "Unlisted" is the option with no Workshop web page, items reachable only through ISteamUGC; it suits an in-game-only library. Making the Workshop public also wants a branding image, title, description and one public item (the page's checklist).
- **Steam Cloud** (Application, Steam Cloud, `/apps/cloud/1309380`): byte quota per user 0, files per user 0, no Auto-Cloud paths. The page won't take Auto-Cloud paths until both quotas are above 0, and quota changes only take effect after a publish. Not set, because where the game saves isn't decided (today it writes `proving_ground_designs.txt` and `design_library.txt` in the folder it runs from). The same page has "Enable cloud support for developers only" for testing.
- **Multiplayer**: nothing to switch on. Valve's Steam Datagram Relay page says that for peer-to-peer on Steam "all you need to do" is use `ISteamNetworkingSockets::CreateListenSocketP2P` and `ConnectP2P`, and Steam takes care of the relays. Lobbies (ISteamMatchmaking) have no settings page. The "Dedicated Servers" page (Application menu) is for server hosting and was left empty.
- **Leaderboards** (Stats & Achievements, Leaderboards, `/apps/leaderboards/1309380`): none exist. The page warns that changes there are committed immediately, with no publish. None made. The planned rating board (a number, highest first, written by the game) can be made on that page as Numeric / Descending, or by the game itself with `FindOrCreateLeaderboard`.

## Left for Henry

- **Name.** The app and store page still say Rock Collector. Renaming is a store-page change, yours whenever you want it.
- **Testers on other accounts.** A second Steam account (for the two-machine test) has to own app 1309380. Keys come from the app's landing page, Technical Tools, Request Steam Product Keys. If testers should also see designs, Workshop visibility needs "Developers & Testers" (with a Steam group) or a wider setting.
- **Steam Cloud quotas and paths**, once the save location is decided. Then publish.
- **The rating leaderboard**, if you want it made on the site instead of by the game.
- **Store page "Supported Features"** ticks (Workshop, Cloud, multiplayer, leaderboards): store-page work for later. The Cloud page says its tick is needed to fully enable Cloud on release.

## What development runs need

- The Steam client running and signed in to an account that owns app 1309380 (Henry's developer account does).
- `steam_appid.txt` holding `1309380` in the folder the game is started from. Steam reads it from the working directory. With `cargo run` from `nomindwing/` that is this folder, the same place the save files land today. Don't ship it in a Steam build: launched by Steam, the game gets the app ID from Steam.
- `steam_api64.dll` from the Steamworks SDK beside the exe. The `steamworks` crate's build is expected to copy it into `target/`; check when the crate goes in.
- Anything that publishes Workshop items works only for accounts allowed by the Workshop visibility above.
