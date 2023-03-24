# compile

Do not use brew to install rust, if you installed rust with brew, you need to uninstall rust first
```
$ brew uninstall rust
```

Install rust using the following command

```
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
$ source $HOME/.cargo/env
```

The path after the installation is complete

```
$ which rustup
$HOME/.cargo/bin/rustup

$ which rustc
$HOME/.cargo/bin/rustc
```

To see the current toolchain and all supported toolchains, use the command

```
$ rustup show
$ rustup target list
$ rustup toolchain list
```

The most common ones are

```
macos with apple m1 m2 chip
aarch64-apple-darwin

macos with intel chip
x86_64-apple-darwin
```

Add appropriate toolchains and delete unused ones

```
This is just an example. The compilation toolchain and compilation target must be determined according to the actual situation.
$ rustup target add aarch64-apple-darwin
$ rustup toolchain install stable-aarch64-apple-darwin
$ rustup target remove x86_64-apple-darwin
$ rustup toolchain uninstall stable-x86_64-apple-darwin 1.67.1-x86_64-apple-darwin
```

Setting up the compilation toolchain

```
This is just an example. The compilation toolchain must be set according to the actual situation.
$ rustup default stable-aarch64-apple-darwin
```

Set up the build platform

```
$ vim $HOME/.cargo/config

This is just an example. The compilation target must be determined according to the actual situation.

Add to

[build]
target = "aarch64-apple-darwin
```

You can also specify at compile time

```
This is just an example. The build target must be set according to the actual situation.
$ cargo build --release --target aarch64-apple-darwin
```
