# rsmc

A stupid little Minecraft clone written in Rust, powered by the Bevy engine.

## Features

* Procedual world generation using 3D Perlin noise
* Custom terrain mesher based on face culling
* Custom Client / Server architecture using Renet
* Data serialization using bincode serde
* World update synchronization between game clients
* World physics using rapier
* World updates using primitive ray casting
* World saving/loading with `.rsmcw` files
* Periodic world backups in `./backups/` directory
* Modular architecture using ECS

## Installation

### Prerequisites

* [Cargo](https://doc.rust-lang.org/cargo/)

### Default setup

Just run the cargo command to install the dependencies and start the game:

```bash
cargo rs # run server
cargo rc # run client
```

Loading world

```bash
car rs -- -w my_world.rsmcw
```

### More optimal setup

Release Builds (for better performance):

```bash
cargo run rs --release
cargo run rc --release
```

Hot reload client

```bash
bin/dev
```

### Installation on NixOS

Nix shell can be used to run the code using the given [Nix Shell Config File](./shell.nix). This will automatically install rust and the bevy dependencies.
Strongly inspired by the [Bevy NixOS installation guide](https://github.com/bevyengine/bevy/blob/latest/docs/linux_dependencies.md)

```bash
nix-shell --run "cargo rs"
nix-shell --run "cargo rc"
```

## Notes

Checkout the [Wiki](https://github.com/cb341/rsmc/wiki) for additional project information.
