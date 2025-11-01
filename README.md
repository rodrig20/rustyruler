# Rustyruler

Rustyruler is a lightweight and efficient ruler tool built with Rust and GTK4. It helps you measure dimensions of elements on your screen by overlaying a dynamic crosshair that automatically detects boundaries based on color changes.

## What it does

Rustyruler makes measuring things on your screen super easy. Just move your mouse around and it'll show you exactly how big an element is. It works by detecting where colors change on your screen to figure out the boundaries of what you're trying to measure.

### Key features

- **Dynamic crosshair**: Red lines follow your mouse and automatically detect where elements start and end
- **Real-time measurements**: See the width and height of whatever you're measuring right away
- **Multiple measurement modes**: Choose between cross, horizontal line, or vertical line depending on what you need to measure
- **Automatic boundary detection**: The tool figures out where elements begin and end by detecting color changes
- **Clean, simple interface**: A control center with visual buttons makes switching between tools a breeze

## Getting Started

### Before you start

Make sure you have:

- [Rust](https://www.rust-lang.org/tools/install) installed
- On Wayland: `grim` (screenshot utility) - you'll need this for taking screenshots
- On X11: You might need some additional Wayland compatibility libraries (not tested)

### Installation

Getting Rustyruler up and running is straightforward:

**Method 1: Build from source**

```bash
# Get the code
git clone https://github.com/rodrig20/rustyruler.git
cd rustyruler

# Build it
cargo build --release

# Or just run it directly
cargo run
```

**Method 2: Install with PKGBUILD (Arch Linux)**

If you're on Arch Linux or an Arch-based distribution, you can use the provided PKGBUILD:

```bash
# Clone the repository
git clone https://github.com/rodrig20/rustyruler.git
cd rustyruler

# Build and install the package
makepkg -si
```

This will build the package and install it using pacman. The PKGBUILD includes all necessary dependencies and will install the binary to `/usr/bin/rustyruler`.

## How to use it

1. Launch the app and you'll see it takes a screenshot of your screen
2. Move your mouse around to start measuring (the crosshair follows you)
3. Press `Ctrl` to show the control center if you want to change tools
4. Pick the measurement mode that works best for what you're measuring:
   - Cross: Full crosshair for measuring both width and height
   - Horizontal Line: Just a horizontal line for width measurements
   - Vertical line: Just a vertical line for height measurements
5. The measurements update in real-time as you move your mouse
6. Press `Escape` when you're done to close the app
