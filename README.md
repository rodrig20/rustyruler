# Rustyruler

A lightweight and efficient ruler tool built with Rust and GTK4. Rustyruler helps you measure dimensions of elements on your screen by overlaying a dynamic crosshair that automatically detects boundaries based on color changes.

## Features

- **Dynamic Crosshair**: Follows your mouse cursor with red lines extending to detect boundaries where color changes occur
- **Real-time Measurements**: Shows the dimensions of the selected area in a tooltip (Width × Height)
- **Automatic Boundary Detection**: Lines extend until a significant color change is detected

## Dependencies

- `gtk4` - GUI toolkit
- `gtk4-layer-shell` - Window layer management
- `image` - Image processing
- `glib` - GLib bindings
- `time` - Time handling
- `grim` - Required external tool for taking screenshots (on Wayland)

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- On Wayland: `grim` (screenshot utility)
- On X11: You may need to install additional Wayland compatibility libraries

## Installation

For Arch-based distros, you can use the provided PKGBUILD:

```bash
makepkg -si
```

Alternatively, you can build from source:

1. Clone the repository:
   ```bash
   git clone https://github.com/rodrig20/rustyruler.git
   cd rustyruler
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Run the application:
   ```bash
   cargo run
   ```

## Usage

1. Launch the application
2. The application will capture a screenshot of your current screen and display it as a fullscreen overlay
3. Move your mouse cursor to measure dimensions of elements on screen
4. The crosshair will automatically extend lines in all 4 directions until it detects a significant color change
5. The dimensions of the bounded area will be displayed in a tooltip near your cursor
6. Press `Escape` to exit the application and clean up temporary files

## How It Works

Rustyruler works by:

1. Capturing a screenshot of the current screen using the `grim` utility
2. Displaying the screenshot as a fullscreen overlay using GTK4 and layer-shell
3. Tracking mouse movements to position the crosshair
4. Analyzing the screenshot pixel by pixel to detect color changes
5. Calculating line limits where color changes exceed a threshold (20.0 in RGB magnitude)
6. Drawing the crosshair and displaying measurements in a tooltip

## Code Structure

- `main.rs`: Application entry point and GTK4 initialization
- `ui.rs`: GUI setup, drawing functions, and event handling
- `screenshot.rs`: Screenshot capture, image loading, and color analysis functions
