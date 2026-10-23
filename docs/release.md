# Releasing Muniko Share

## Cut a release
1. Bump `version` in `apps/desktop/src-tauri/tauri.conf.json`, `apps/desktop/src-tauri/Cargo.toml` and `apps/desktop/package.json`.
2. Tag and push: `git tag v0.2.0 && git push origin v0.2.0`.
3. `.github/workflows/release.yml` builds Linux (`.AppImage`, `.deb`, `.rpm`), macOS (universal `.dmg` and `.app`) and Windows (`.msi`, NSIS `.exe`), signs the updater artifacts and creates a **draft** GitHub release with `latest.json`. Review it, then publish.

## Release files
Assets are named `MunikoShare-<version>-<platform>-<arch>[-setup].<ext>` through `releaseAssetNamePattern` in `release.yml`; users only need these:

| Platform | File | Use |
|---|---|---|
| Windows | `*-windows-x64-setup.exe` | Normal installer |
| Windows | `*-windows-x64.msi` | IT and silent installs |
| macOS | `*-macos-universal.dmg` | Intel and Apple Silicon |
| Linux | `*.AppImage` | Any distro, no install |
| Linux | `*.deb` / `*.rpm` | Debian-based / Fedora-based |

`latest.json`, the `.sig` files and the macOS `.app.tar.gz` exist only for the auto-updater.

## Updater keys (one time)
1. `npx tauri signer generate -w ~/.tauri/muniko-share.key` (run it in `apps/desktop`).
2. Put the **public** key into `plugins.updater.pubkey` in `apps/desktop/src-tauri/tauri.conf.json` (currently a placeholder).
3. Store the private key and its password as repository secrets (below). Never commit the key.
4. The endpoint is `https://github.com/Ayesh96/MunikoShare/releases/latest/download/latest.json`; change `plugins.updater.endpoints` if the repository moves.

Updater artifacts are only produced in CI: `src-tauri/tauri.release.conf.json` turns on `bundle.createUpdaterArtifacts`, so local builds need no key.

## Repository secrets
| Secret | Used for |
|---|---|
| `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Signing updater artifacts (required for updates) |
| `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY` | macOS code signing (base64 `.p12`, its password, identity name); also add them to the `env` of the tauri-action step, because empty values break the build |
| `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID` | macOS notarization (app-specific password) |
| Windows certificate | Not wired yet; add `bundle.windows.certificateThumbprint` or a signing command once a certificate exists. Unsigned builds trigger SmartScreen. |

Apple and Windows signing are not wired into `release.yml` yet, so those builds are unsigned.

## OS integration
- **Windows:** the NSIS installer hook (`src-tauri/installer/hooks.nsh`) creates and removes `%APPDATA%\Microsoft\Windows\SendTo\Muniko Share.lnk` (arguments `--send`). The app also rewrites that shortcut on each start (`os_integration.rs`), which covers MSI installs and moved installs. The shortcut is not removed by the MSI uninstaller.
- **Linux:** the `.deb` and `.rpm` use `src-tauri/installer/desktop.template` (`Exec=... %F`, `MimeType=application/octet-stream;inode/directory;`). AppImage relies on the generated desktop entry and file associations.
- **macOS:** `bundle.fileAssociations` declares `public.data` and `public.folder` with role Viewer, so "Open With" works; files opened while running arrive through `RunEvent::Opened`.
- **Any OS:** `muniko-share [--send] <paths...>`; a second launch forwards its paths to the running instance, which opens the "Send to..." picker.

Windows and Linux integration are untested on real machines; only macOS was built and run locally.
