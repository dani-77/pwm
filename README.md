<h1 align="center">
  <img src="logo.png" alt="pwm logo" width="160"><br>
  PWM

  Pipa Window Manager
</h1>

A minimalistic, yet complete, Penrose X11 Tiling Window Manager that is light and fun to use.

Named in honor of my cat Pipa; an adorable tortoiseshell cat who was with me for almost 15 years and whose name is also the Brazilian word for paper kite;

The P came from diverse places, but the Penrose library couldn't be forgotten. 

The WM you all know from where it came.

## Build Dependencies

- cargo & rust

- libX11-devel, libXft-devel

- make

## Running Dependencies

- dbus

- dmenu

- st

- wireless_tools

- [JetBrainsMono Nerd Font](https://www.nerdfonts.com/) (e.g. Void's `nerd-fonts-otf` package)

  The status bar (`src/bar.rs`) renders battery, CPU, RAM, volume and layout
  indicators as icon glyphs from the Nerd Fonts private-use-area ranges (e.g.
  `\u{f2db}` for the CPU chip icon). Every built-in theme (see
  [Configuration](#configuration) below) defaults to the `JetBrainsMono Nerd
  Font` family; without it installed, those icons render as blank space
  instead of a crash, since a missing glyph just draws nothing. If you want a
  different font, set `font` under your theme in `config.toml` to another
  Nerd Font family available on your system.

### Optional dependencies

- dunst

- feh

- redshift

- scrot

- slock

- sxhkd

- udiskie

- xautolock

- xcompmgr

# Build / Install

To build and use locally:

```
$ make
```

And then run the package:

```
$ target/release/./pwm
```

To install widely:

```
$ sudo make install
```

## Configuration

pwm reads `~/.config/pwm/config.toml` (or `$XDG_CONFIG_HOME/pwm/config.toml`)
once at startup. It's entirely optional - anything you leave out, or the
whole file if you never create one, keeps pwm's built-in defaults, and a
mistake in it falls back safely with a warning rather than stopping pwm from
starting. There's no hot reload yet, so changes need a logout/login (or a
fresh `startx`) to take effect.

A fully-commented example, listing every available option, is installed to
`/usr/share/pwm/config.toml.example` - copy it to get started:

```
$ mkdir -p ~/.config/pwm
$ cp /usr/share/pwm/config.toml.example ~/.config/pwm/config.toml
```

What you can configure:

- **`[theme]`** - pick one of the built-in themes (`d77`, `gruvbox_dark`,
  `arc_dark`, `dracula`, `tokyo_night`) via `active = "..."`, or define your
  own colors and font under `[theme.<name>]`.
- **`[apps]`** - which programs `terminal`/`launcher`/`locker` spawn, and
  where `startup_script` lives.
- **`[[window_rule]]`** - which window class opens on which tag. Defining
  any rules here replaces the built-in list rather than adding to it.
- **`[bar]`** - `widgets`, an ordered list picking which bar widgets show
  (from `workspaces`, `layout`, `window_name`, `cpu`, `ram`, `volume`,
  `wifi`, `battery`, `clock`).
- **`[[bind]]`** - remap keybindings entirely. Like window rules, any
  `[[bind]]` entries replace the whole built-in keymap - keep a way to spawn
  a terminal and to `exit` among them, or you may need to switch to another
  VT to recover. See the example file for the full list of actions.

## Keybinds

The keybindings below are the built-in defaults, fully configurable via
`config.toml`'s `[[bind]]` (see [Configuration](#configuration) above):

super + return -> st (suckless terminal)

super + d -> dmenu (suckless menu)

super + shift + f -> full screen toggle

super + j/k -> swap focused window

super + shift + j/k -> swap position focused window

super + m -> change layout

super + q -> kill focused window

super + s -> scratchpad toggle

super + x -> session menu

super + shift + q -> quit WM

# Credits

- Huge thanks to [sminez](https://github.com/sminez) for the fantastic Penrose Library, examples and HowTo videos in Youtube.


Happy hacking!

