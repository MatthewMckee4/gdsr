# gdsr
GDSII manipulation, written in rust.

## Documentation

The documentation for this project is available at [matthewmckee4.github.io/gdsr/](https://matthewmckee4.github.io/gdsr/).

## Installation

I recommend using [uv](https://github.com/astral-sh/uv) to manage your python packages.

To install and use yourself:

```bash
uv pip install gdsr
```

To use from source code:

```bash
uv pip install requirements-dev.txt

maturin develop
# or
uv pip install .
```

## What can you do with gdsr

gdsr offers many features which include but are not limited to:
- Easy reading from and writing to gds files
- Strictly typed python code
- Easy to understand code

## Inspiration

My main inspiration comes from [gdstk](https://github.com/heitzmann/gdstk). If you are looking for an extremely fast gds manipulation python package then i would strongly recommend heading over and having a look at his work.

Other inspirations include:
- [gdsfactory](https://github.com/gdsfactory/gdsfactory)
- [klayout](https://www.klayout.org/klayout-pypi/)

## How to get started using gdsr

A simple program below shows the easy to use interface.

```python
import gdsr

library = gdsr.Library("My First Library")

cell = gdsr.Cell("My First Cell")

cell.add(gdsr.Text("Hello, World!"))

library.add(cell)

library.to_gds("My first gdsr output.gds")
```

## Need help?

Head over to the [discussions page](https://github.com/MatthewMckee4/gdsr/discussions) and create a new discussion there or have a look at the [issues page](https://github.com/MatthewMckee4/gdsr/issues) to see if anyone has had the same issue as you.