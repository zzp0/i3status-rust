# i3status-rust

![demo1](https://raw.githubusercontent.com/greshake/i3status-rust/master/img/example_bar.png)

`i3status-rs` is a feature-rich and resource-friendly replacement for i3status, written in pure Rust. It provides a way to display "blocks" of system information (time, battery status, volume, etc) on the [i3](https://i3wm.org/) bar. It is also compatible with [sway](http://swaywm.org/).

For a list of available blocks, see the [block documentation](blocks.md). Further information can be found on the [Wiki](https://github.com/greshake/i3status-rust/wiki).

## Requirements

The Rust language and the `cargo` package manager are required to build the binary.

We also require Libdbus 1.6 or higher. On some older systems this may require installing `libdbus-1-dev`. See [#194](https://github.com/greshake/i3status-rust/issues/194) if you are having dbus-related compilation issues.

Compilation is only tested with very recent stable versions of `rustc`. If you use a distro with older Rust packages, consider using [rustup](https://rustup.rs/) to install a newer toolchain.

Most blocks assume you are running Linux, and some have their own system requirements; these are mentioned in the [block documentation](blocks.md).

Optional:

* Font Awesome 4.x is required when using the icons config `name = "awesome"`. For version 5, use `name = "awesome5"`. On Arch Linux version 4 is available in the [`AUR`](https://aur.archlinux.org/packages/ttf-font-awesome-4/), and version 5 is available [`here`](https://www.archlinux.org/packages/community/any/ttf-font-awesome/).
* Powerline Fonts are required for all themes using the powerline arrow char.
* `gperftools` is required for building with the `"profiling"` feature flag (disabled by default).

## Getting Started

Stable releases are packaged on some distributions:

* On Arch Linux: `sudo pacman -Syu i3status-rust`. The latest development version can be installed from the [AUR](https://aur.archlinux.org/packages/i3status-rust-git).

* On Fedora 31+: `sudo dnf install i3status-rs`. For older releases and CentOS, you can install from the [COPR](https://copr.fedorainfracloud.org/coprs/atim/i3status-rust/).

* On Void Linux: `xbps-install -S i3status-rust`

* On NixOS: `nix-env -iA nixos.i3status-rust`

Otherwise, you can install from source:

```shell
$ cargo install --git https://github.com/greshake/i3status-rust i3status-rs
```

By default, this will install the binary to `~/.cargo/bin/i3status-rs`.

or manually:

```shell
$ git clone https://github.com/greshake/i3status-rust
$ cd i3status-rust && cargo build --release
# Optional:
$ cp target/release/i3status-rs ~/bin/i3status-rs
```

## Configuration

After installing `i3status-rust`, you need to create a configuration file.
Edit the [example configuration](https://raw.githubusercontent.com/greshake/i3status-rust/master/example_config.toml) to your liking and put it to a sensible place, such as `~/.config/i3/status.toml`.

There are some top-level configuration variables:

Key | Description | Required | Default
----|-------------|----------|--------
`icons` | The icon set that should be used. Possible values are `none`, `awesome`, `awesome5` and `material`. Check [themes.md](https://github.com/greshake/i3status-rust/blob/master/themes.md) for more information | No | `none`
`theme` | The predefined theme that should be used. You can also add your own overrides. Check [themes.md](https://github.com/greshake/i3status-rust/blob/master/themes.md) for all available themes. | No | `plain`
`scrolling` | The direction of scrolling, either `natural` or `reverse` | No | `natural`
`block` | All blocks that will exist in your i3bar. Check [blocks.md](https://github.com/greshake/i3status-rust/blob/master/blocks.md) for all blocks and their parameters. Don't forget about the [example configuration](https://raw.githubusercontent.com/greshake/i3status-rust/master/example_config.toml). | No | none

## Integrate it into i3

Next, edit your i3 bar configuration to use `i3status-rust`. For example:

```text
bar {
    font pango:DejaVu Sans Mono, FontAwesome 12
    position top
    status_command path/to/i3status-rs path/to/your/config.toml
    colors {
        separator #666666
        background #222222
        statusline #dddddd
        focused_workspace #0088CC #0088CC #ffffff
        active_workspace #333333 #333333 #ffffff
        inactive_workspace #333333 #333333 #888888
        urgent_workspace #2f343a #900000 #ffffff
    }
}
```

In order to use the built-in support for the Font Awesome icon set, you will need to include it in the `font` parameter, as above. Check to make sure that "FontAwesome" will correctly identify the font by using `fc-match`, e.g.

```shell
$ fc-match FontAwesome
fontawesome-webfont.ttf: "FontAwesome" "Regular"
```

Note that the name of the Font Awesome font may have changed in version 5.  
You can use `fc-list` to see the names of your available Awesome Fonts.

```shell
$ fc-list | grep -i awesome
/usr/share/fonts/TTF/fa-solid-900.ttf: Font Awesome 5 Free,Font Awesome 5 Free Solid:style=Solid
/usr/share/fonts/TTF/fa-regular-400.ttf: Font Awesome 5 Free,Font Awesome 5 Free Regular:style=Regular
```

In this example, you have to use `Font Awesome 5 Free` instead of the `FontAwesome 12` in the example configuration above.
You can verify the name again using `fc-match`

See [#130](https://github.com/greshake/i3status-rust/issues/130) for further discussion.

Finally, reload i3: `i3 reload`.

## Contributing

We welcome new contributors! Take a gander at [CONTRIBUTING.md](CONTRIBUTING.md).

## License

This project is licensed under the GPLv3. See the [LICENSE.md](LICENSE.md) file for details.
