# sgmenu

`sgmenu` is a command launcher that fulfills a similar purpose to programs
such as `dmenu` or `rofi`. It is targeted at the `Sway` window manager in
Subgraph OS.

## Features 

Some special features are supported that make it different from other launchers.
Namely:

- It uses `GIO` library to find `.desktop` files and parse them
- It can wrap commands in a `pty`, this is especially useful for launching
commands in `systemd-nspawn` via machinectl
- Custom `dmenu` commands are support (current default is to run `rofi` in 
dmenu mode)

## Disclaimer

`sgmenu` may not be generally useful or **safe** for other purposes. It is also
likely to change drastically to better support the `Sway` window manager in 
Subgraph OS.  

# Using sgmenu

The main intended use is to provide a working desktop launcher for the `Sway`
window manager in Subgraph OS. In this particular case, `.desktop` files are
gathered by the [desktopd](https://github.com/subgraph/desktopd). 


## Examples

Launching the default `dmenu`-like command to select an application to run:
```bash
$ sgmenu -d
```

Launching a custom `dmenu`-like command:
```bash
$ sgmenu -d 'dmenu'
```

Listing `.desktop` applications:
```bash
$ sgmenu -l
```

Launching any command via a pty wrapper:
```bash
$ sgmenu -c 'ls -la'
```

