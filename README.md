# wasp-launcher

wasp-launcher is a tauri app and is a launcher for WaspScripts scripts.

Start dev environment:

```cmd
pnpm tauri dev
```

Update types (needs auth):

```cmd
pnpm gentypes
```

Build tauri app:

```cmd
pnpm tauri build
```

## Installing on Linux

Releases include three Linux downloads: an AppImage, a `.deb` and a plain binary (`wasp-launcher-linux-x86_64.tar.gz`).

### Plain binary (recommended on Arch and other rolling distros)

The binary uses the WebKitGTK installed on your system, so you need to install it first:

| Distro | Packages |
| --- | --- |
| Arch / Manjaro / Omarchy | `sudo pacman -S webkit2gtk-4.1 gtk3` |
| Debian / Ubuntu | `sudo apt install libwebkit2gtk-4.1-0 libgtk-3-0` |
| Fedora | `sudo dnf install webkit2gtk4.1 gtk3` |

Then install it with the install script:

```sh
curl -fsSL https://raw.githubusercontent.com/WaspScripts/wasp-launcher/main/scripts/install-linux.sh | bash
```

This puts the binary in `~/.local/bin/wasp-launcher`, adds an app menu entry and icon, and installs an update command next to it. Make sure `~/.local/bin` is in your `PATH`.

#### Updating

The binary does not update itself. Run the update command, which downloads the latest release if there is a newer one:

```sh
wasp-launcher-update
```

Add `--force` to reinstall the current version. To use a shorter name, add an alias:

```sh
# bash / zsh: add to ~/.bashrc or ~/.zshrc
alias wasp-update='wasp-launcher-update'

# fish: run once
alias --save wasp-update 'wasp-launcher-update'
```

If WaspLauncher is open while you update, close and reopen it to use the new version.

Prefer to do it by hand? Download `wasp-launcher-linux-x86_64.tar.gz` from the [latest release](https://github.com/WaspScripts/wasp-launcher/releases/latest) and extract it with `tar -xzf wasp-launcher-linux-x86_64.tar.gz -C ~/.local/bin`.

### AppImage

Works on most distros, but it bundles its own WebKitGTK and can crash on startup with `Could not create surfaceless EGL display` on distros with a newer graphics stack (e.g. Arch). If that happens, use the plain binary instead.

### Debian / Ubuntu package

```sh
sudo apt install ./wasp-launcher.deb
```

### Plugins

RemoteInput needs `patchelf` 0.18 or newer on Linux (`sudo pacman -S patchelf` / `sudo apt install patchelf`).
