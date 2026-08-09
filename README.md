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

## Keybinds

Eventhough you can swap any of it, by default:

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

