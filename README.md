<div align="center">

**English** · [Español](README.es.md)

<img src="docs/marca/banner.png" alt="Jiyu Launcher" width="100%">

<p>
<img src="https://img.shields.io/badge/Rust-iced%200.13-F4511E?style=flat-square&logo=rust&logoColor=white" alt="Rust + iced">
<img src="https://img.shields.io/badge/Linux-x86__64-0E0E10?style=flat-square&logo=linux&logoColor=white" alt="Linux x86_64">
<img src="https://img.shields.io/badge/engine-Horizon-F4511E?style=flat-square" alt="Engine: Horizon">
<img src="https://img.shields.io/badge/license-MIT-0E0E10?style=flat-square" alt="MIT license">
<img src="https://img.shields.io/badge/telemetry-none-2E7D32?style=flat-square" alt="No telemetry">
</p>

**A Minecraft launcher laid out like Prism Launcher, for playing Horizon (Minecraft rewritten in Rust) and classic Minecraft Java side by side.**

About 50 MB of memory at idle, 0 % CPU, and it steps aside while you play.

[Install](#install) · [Usage](#usage) · [Themes](#themes) · [Jiyu ecosystem](#the-jiyu-ecosystem) · [FAQ](#faq)

</div>

![The main window with the Jiyu Pulse theme](docs/capturas/principal.png)

> [!NOTE]
> The launcher's interface is in **Spanish** for now. Menu names below are given in Spanish with their English meaning, e.g. *Ajustes* (Settings). An English interface is planned.

## Features

- **All your instances in one window.** Horizon (Rust) instances, the Java instances Horizon creates, and Prism Launcher instances, read as they are without changing anything in Prism. A badge and a colored edge tell Rust from Java at a glance.
- **A cleaner interface.** Prism's familiar layout, redrawn: smooth vector icons, rounded cards, an instance panel with play time and last played, real right-click menus, and four themes. Every list is drawn to stay fast with hundreds of instances.
- **Three views:** groups (with Prism's divider lines), Nautilus-style folders that can live inside other folders, and a list. Colored folders, manual drag-to-sort, favorites, and an archive for what you don't want to see, including whole groups.
- **Mods, shaders and resource packs from Modrinth** in one click. Every download goes to a queue that keeps going while you search for something else. What the instance already has shows as *Installed* (matched by file hash), and required dependencies are fetched automatically. On Rust instances, Horizon ports the Java mod and tells you whether it runs fully or partly.
- **Modpacks:** create an instance straight from a Modrinth modpack, or copy a Prism instance into Horizon.
- **Big Picture:** a full-screen couch mode with shelves (*Keep playing* and one per folder) that you browse with the arrow keys. `F11` toggles it.
- **Horizon native mods:** install them from the launcher and build them with one button per instance, with live progress.
- **Jiyu in game:** the 16 px Jiyu logo in the bottom-left corner of Horizon, and as the game window's icon. Both optional.
- **Wardrobe:** your skins in 3D that you can spin, your account's skin and capes, and a different skin per instance that is applied automatically when you play it.
- **Servers:** add them once and they show up in the Multiplayer list of every instance, with live status (players, ping and MOTD).
- **Performance profiles:** GameMode, MangoHud, dedicated GPU, NVIDIA tweaks, Java memory and garbage collector, huge pages, all per PC profile.
- **Themes:** Jiyu Pulse, Matugen, Prism Dark and Prism Bright, with a corner mascot (Prism's cat or Jiyu's) you can turn off.
- **Lightweight:** software rendering by default so it never fights the game for the GPU. When you play, it closes and leaves a 4 MB watcher that reopens it when you're done.
- **One-command install** that sets up system packages, Rust and Horizon (verified by SHA256) for you.

**Coming next**

- **Minecraft Legacy Edition instances**, alongside Horizon, Java and Prism.
- An English interface.
- Gamepad support in Big Picture.
- Launching Java instances (vanilla, Fabric, Quilt) without Horizon.

| Big Picture | Browse mods |
|---|---|
| ![Big Picture](docs/capturas/big-picture.png) | ![Browse mods](docs/capturas/explorar.png) |
| **Horizon native mods** | **Servers** |
| ![Native mods](docs/capturas/mods-nativos.png) | ![Servers](docs/capturas/servidores.png) |
| **Folders** | **Settings** |
| ![Folders](docs/capturas/carpetas.png) | ![Settings](docs/capturas/ajustes.png) |

## Themes

Four themes that switch instantly in *Ajustes → Apariencia* (Settings → Appearance). The main one is **Jiyu Pulse**: sumi (near black) with the Jiyu brand's shu vermilion, the Jiyu mascot in the corner and rounded buttons.

| Jiyu Pulse | Matugen |
|---|---|
| ![Jiyu Pulse theme](docs/capturas/temas/jiyu-pulse.png) | ![Matugen theme](docs/capturas/temas/matugen.png) |
| **Prism Dark** | **Prism Bright** |
| ![Prism Dark theme](docs/capturas/temas/prism-dark.png) | ![Prism Bright theme](docs/capturas/temas/prism-bright.png) |

## The Jiyu ecosystem

Jiyu Launcher is part of **Jiyu**, a family of free apps made by one person, sharing one brand (the pulse, shu vermilion on sumi, and the mascot) and the same rules: no ads, no accounts of its own, no telemetry.

| | App | What it is | Status |
|---|---|---|---|
| <img src="assets/logo/jiyu-16.svg" width="32" alt=""> | **Jiyu Launcher** | This Minecraft launcher for Horizon, Java and Prism, written in Rust | In development, Linux |
| <img src="assets/logo/jiyu-pulse.svg" width="32" alt=""> | **Jiyu Pulse** | A music app for iPhone and Android: free, no ads, no account, with downloads, lyrics, Jam between phones and players with personality (cassette, vinyl) | In development |

Jiyu Pulse draws the brand's pulse with a smooth stroke. Jiyu Launcher draws it in pixels, 16×16, like one more Minecraft block <img src="assets/logo/jiyu-16.svg" width="16" alt="">: the same logo in the launcher and inside the game.

## Install

Linux only (tested on Arch/CachyOS, x86_64).

```sh
git clone https://github.com/ToxBox8/jiyu-launcher.git
cd jiyu-launcher
./instalar.sh
```

The installer does everything, and asks before installing anything system-wide:

1. **Missing system packages:** git, curl, a C compiler and the windowing libraries (Wayland, X11, xkbcommon, Vulkan). Uses pacman, apt, dnf or zypper.
2. **Rust**, through rustup and only for your user, if you don't have `cargo`.
3. **Builds** the launcher and installs it to `~/.local/bin/jiyu-launcher`, with an entry in your app menu.
4. **Horizon**, if you don't have it: downloads the latest official release from [horizon-rust/horizon](https://github.com/horizon-rust/horizon), checks its SHA256 and puts it in `~/Applications/horizon-launcher-linux-x86_64/`.

Options:

| Command | What it does |
|---|---|
| `./instalar.sh --si` | Same, without asking |
| `./instalar.sh --sin-horizon` | Doesn't download Horizon (you'll only see your Prism instances) |
| `./instalar.sh --desinstalar` | Removes the launcher; your settings, skins and instances stay |

**To update:** `git pull` and run `./instalar.sh` again.

### What you need to play

- **A Microsoft account that owns Minecraft: Java Edition.** Sign in from *Cuentas* (Accounts), through Horizon.
- **Horizon**, for Rust instances and to launch Java ones (it downloads Minecraft, Java and mod loaders itself). The installer adds it if missing.
- **Prism Launcher is not required.** If you have it, its instances show up too and are launched with Prism.

## Usage

### Instances

- **Click** selects, **double-click** plays. The right panel shows the selected instance; click its icon to change it.
- **Right-click** an instance: Play, Edit, Change icon, Mods, Download mods/shaders/packs, Skin on launch, Move to folder, Open folder, Archive and Delete. Horizon instances also get Duplicate and Desktop shortcut; Prism instances get Copy to Horizon.
- **New instance** (`Ctrl+N` or *Añadir instancia*): Horizon (Rust), Java (vanilla, Fabric, Quilt, Forge or NeoForge), copied from Prism, or from a Modrinth modpack. Pick icon, folder, memory and skin right there.
- **Delete** asks you to type `BORRAR`. Prism instances go to the trash instead of being erased.

### Folders and groups

- Three views, top right: **Grupos** (groups, like Prism), **Carpetas** (folders, like Nautilus) and **Lista** (list).
- Folders can be nested («1.21 › mods»). Drag an instance onto a folder to move it.
- Right-click a folder: new instance inside, new folder inside, **color** (9 colors), rename, **archive the group**, or remove the folder (its contents move up a level).
- **Sorting:** by name, last played, most played, or **manual**: drag an instance onto another.
- **Archive** (bottom right): archived items don't show in the window or in search, but keep their worlds. You can play from inside it and restore whenever you want, single instances or whole groups.

### Downloading mods, shaders and resource packs

Open it from *Explorar mods* (Browse mods), right-click → *Descargar…*, or from an instance's *Editar → Mods / Paquetes de recursos / Shaders* (already pointed at that instance).

- On the left you pick **where to install**, **what you're looking for** (mods, resource packs, shaders, modpacks or data packs) and **from where** (Modrinth, Horizon or your favorites).
- Every result has its own **Install** button. You can also **tick several** (checkbox or double-click the row) and install them together. Ticks are kept across searches and types.
- Everything goes to a **download queue** that keeps going while you change search, type or instance. The button changes to *Queued*, *Downloading…* and *✓ Installed*, or *Retry* if it fails. *Descargas* in the footer shows the queue with each error.
- What the instance **already has** shows as *Installed*, matched by each file's SHA-1 on Modrinth.
- **Required dependencies** are downloaded automatically for the instance's Minecraft version and loader.
- **Filters:** loader and «only for your version». If nothing exists for your version (common with snapshots), it shows any version and tells you.
- More results load as you reach the end of the list.
- The **star** saves a project to your favorites.
- Putting a mod into an instance without Fabric or Forge, or a data pack outside a world, gets a clear warning instead of a broken install.

How it installs per instance type:

| Instance | How it's installed |
|---|---|
| Prism | Straight from Modrinth into its `mods/`, `resourcepacks/` or `shaderpacks/` folder |
| Horizon Java | Installed by Horizon, which verifies every file's hashes |
| Rust (Horizon) | Horizon **ports the Java mod** to Horizon. The list tells you whether it runs fully or partly |
| Shaders anywhere | Straight from Modrinth. Rust instances use Horizon's own shaders: pick the *Horizon (nativos)* source |

### Horizon native mods (building)

**Native mods** (`.hnmod`, from the *Horizon (nativos)* source) don't load until the game is built with them.

1. Install the native mods you want.
2. Go to the Rust instance's **Editar → Mods nativos** and press **Compilar mods** (Build mods).
3. Confirm. These mods run with full access to your computer, like any program: only build them if you trust their authors. Mods from Horizon's registry are reviewed; ones installed from GitHub are not.
4. The first time, Horizon downloads its own Rust toolchain and takes **20–30 minutes**; after that, a few minutes. Progress is shown live, you can close the window, and you get notified when it's done.

Build again every time you install, remove or update a native mod. You can also get here from *Explorar* («Ir a compilar») and from Big Picture.

### Big Picture

For playing from the couch: `F11` or the *Big Picture* toolbar button.

- The selected instance is shown large at the top, with **Play**, **Edit** and **Mods** (and **Build mods** on Rust instances).
- Below are the **shelves**: *Seguir jugando* (Keep playing, your last 10) and one per folder. Move with the arrow keys or the mouse wheel.
- `Enter` plays, `E` edits, `Esc` or `F11` exits. Arrows only move, they never launch. With the mouse, one click selects and another click on the selected card plays.
- Only what fits on screen is drawn. «‹ 2» or «3 ›» show there are more cards on that side, and «Estante 3 de 6» (shelf 3 of 6) that there are more shelves.

### Wardrobe and skins

- Your skins in **3D**: drag to spin them. Import more from a file.
- Change your Minecraft **account skin and cape** from here.
- **Per-instance skin** (*Editar → Skin*): your current one, one from the wardrobe or an imported one. It's uploaded to your account right before that instance launches.
- Your skin's head appears next to your name, top right.

### Servers

- Add servers (name and address) in *Servidores*. They appear **in the Multiplayer list of all your instances**, Horizon and Prism.
- They're added before launching, or right away with *Ponerlos ya en todas* (add to all now). Servers each instance already had are kept, and a `servers.dat_old` backup is saved.
- See whether each server is online without opening the game: players, ping, MOTD and icon.

### Settings

| Page | What's in it |
|---|---|
| Rendimiento (Performance) | Profiles for your PC, close the launcher while playing, GameMode, MangoHud, dedicated GPU, NVIDIA tweaks, Java memory and GC, huge pages, *Dibujar con la gráfica* (GPU rendering) and Horizon's integrated server on its own thread |
| Apariencia (Appearance) | Theme, Rust/Java badge, colored edge, hexagon for Rust, label under the name, corner mascot, icon size and whether to show Prism instances |
| Horizon | Native mods (build profile and cache limit), Jiyu in game (logo and window icon), versions (auto-install new ones, remove old ones, how many to keep) and game arguments |
| Red y sistema (Network & system) | Proxy (http or socks5, used by Horizon and by the launcher's downloads), reopen the launcher when the game closes, cache sizes and the shortcut list |
| Avanzado (Advanced) | Where Horizon, Prism and your settings live |

*Dibujar con la gráfica* (GPU rendering, under Performance) makes the window smoother (especially full-screen Big Picture) but uses about 150 MB instead of 50. It applies the next time you open the launcher.

### Jiyu in game (Horizon)

In *Ajustes → Horizon → Jiyu dentro del juego*:

- **Jiyu logo in game:** the 16 px pulse in the bottom-left corner, in game, in menus and on the title screen. `F1` hides it along with the rest of the HUD, and it can be turned off in the game itself (*Options → Mods → Jiyu Logo*). It's a tiny native mod (`horizon-mod/`, bundled with the launcher) that is added when you press **Compilar mods** in *Editar → Mods nativos*, alongside your other native mods. Rust instances only.
- **Jiyu icon on the game window:** the 16 px logo as Horizon's icon in the taskbar and window switcher. It changes the `Icon=` line of `~/.local/share/applications/horizon.desktop` and re-applies it before every launch, in case Horizon rewrites that file. Turning it off restores Horizon's icon.

Java instances can't show the logo in game: that would need a Fabric or Forge mod for every version.

## Keys

| Key | Normal window | Big Picture |
|---|---|---|
| `Enter` | Play the selected instance | Play |
| `← → ↑ ↓` | | Move across shelves |
| `E` | | Edit |
| Mouse wheel | | Change shelf or card |
| `Esc` | Close the dialog or menu | Leave Big Picture |
| `F11` | Enter Big Picture | Leave |
| `Ctrl+F` | Search instances | |
| `Ctrl+N` | New instance | |
| `Delete` | Delete (asks you to type BORRAR) | |
| `F5` | Reload | |

## FAQ

**Do I need Horizon?**
Yes, to play: it launches Rust instances and Java ones (downloading Minecraft, Java and loaders itself) and handles the Microsoft sign-in. The installer adds it if missing. Without Horizon you'll only see your Prism instances.

**And Prism Launcher?**
Not needed. If you have it, its instances show up in the window and are launched with Prism. They're only written to when you install mods or delete them.

**Does Essential work on Rust instances?**
No. Horizon analyzes it (`horizon-launcher mods check essential`) and answers *Doesn't run*. Essential's file on Modrinth is a two-class loader that, once Java starts, downloads the real (closed) mod from Essential's servers. Horizon can only port what's inside the file, and there's nothing there. To play with friends on Horizon, use its own features (open to friends, emotes) or a Java instance with Essential.

**Why won't a mod download?**
Open *Descargas* in Browse's footer: each failure says why. Usually there's no build for your Minecraft version or loader, or the instance has no mod loader.

**Can it depend less on Horizon?**
Downloads already don't (except on Horizon's own instances, where it works well and also ports mods to Rust). The next step is launching Java instances on its own (vanilla, Fabric and Quilt first), getting Java from Adoptium or Mojang. Sign-in would need its own Azure app approved by Mojang. Rust instances will always need Horizon, because Horizon is the game itself.

## Known issues

- **Logo in game:** the mod is written against your Horizon's native mod API (nightly 11-08, API 410) and its tests pass, but it hasn't been built inside the game yet. If the build fails, the error shows in *Editar → Mods nativos*.
- **Gamepad:** Big Picture works with keyboard and mouse; gamepads aren't supported yet.
- **Fonts:** no Chinese, Japanese or Korean glyphs are bundled; those mod names show as boxes.
- **Software rendering:** hover highlighting in menus only works with *Dibujar con la gráfica*. Without it, it left artifacts when repainting.
- **From RustCraft to Jiyu:** if you played with the version from before the rename, an old watcher (`rustcraft --vigilar`) may still be waiting for the game to close. Stop it with `pkill -f 'release/rustcraft --vigilar'`.

## Where it stores things

| What | Where |
|---|---|
| Settings | `~/.config/jiyu-launcher/ajustes.json` |
| Skins and game logs | `~/.local/share/jiyu-launcher/` |
| Mod logos, icons and Prism cats | `~/.cache/jiyu-launcher/` (safe to delete) |
| Horizon instances | Horizon's own, in `~/.local/share/horizon/` |
| Prism instances | Wherever Prism keeps them; only written to when installing mods or deleting |

If you're coming from when it was called RustCraft, the `rustcraft` folders are renamed automatically the first time you open it.

## Privacy

No telemetry. It only connects to Modrinth (search, downloads and logos), GitHub (Prism's stock icons), the Minecraft API (your skin and capes) and the servers you add, to check their status. Your account token is requested from Horizon for a single use, discarded right away and passed to `curl` through standard input: it's never stored anywhere and never shows up in the process list.

## Development

```sh
cargo build --release
./target/release/jiyu-launcher
```

Code structure, performance numbers and how to take screenshots headlessly are in [docs/DESARROLLO.md](docs/DESARROLLO.md) (Spanish). To contribute, see [CONTRIBUTING.md](CONTRIBUTING.md); changes per version are in [CHANGELOG.md](CHANGELOG.md).

The logo's native mod lives in [`horizon-mod/`](horizon-mod/); its tests run with `rustc --test horizon-mod/mixins/jiyu_logo/runtime.rs`.

## License

The code is MIT (see [LICENSE](LICENSE)). The Inter typeface ships under its OFL license (`assets/fonts/Inter-LICENSE.txt`). The Jiyu logo and mascot belong to the Jiyu brand and are not covered by the MIT license.

Jiyu Launcher is not an official Minecraft product and is not approved by or associated with Mojang or Microsoft. Horizon and Prism Launcher are separate projects.
