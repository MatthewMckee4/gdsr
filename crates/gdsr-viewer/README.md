# gdsr-viewer

An interactive GDS file viewer built with [egui](https://github.com/emilk/egui).

## Features

- Open and render GDSII files with colored polygons, paths, and text
- Pan and zoom navigation
- Layer visibility toggles with color coding
- Cell selector dropdown

## Prerequisites

On Linux, install the required system libraries:

```sh
sudo apt-get install -y libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
```

No extra dependencies are needed on macOS or Windows.

## Usage

```sh
cargo run -p gdsr-viewer
```

Use **File → Open** to load a `.gds` file. Pan by dragging, zoom with scroll wheel.
