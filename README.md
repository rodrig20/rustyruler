# Rustyruler

A lightweight and efficient ruler tool built with Rust and GTK4. Rustyruler helps you measure dimensions of elements on your screen by overlaying a dynamic crosshair that automatically detects boundaries based on color changes.

## Features

- **Dynamic Crosshair**: Follows your mouse cursor with red lines extending to detect boundaries where color changes occur
- **Real-time Measurements**: Shows the dimensions of the selected area in a tooltip (Width × Height)
- **Multiple Measurement Modes**: Three different tools available via toggle buttons:
  - Cross tool: Measures both width and height (full crosshair)
  - Horizontal line tool: Measures width only (horizontal line)
  - Vertical line tool: Measures height only (vertical line)
- **Toggle Tool Selection**: Use the control center with image-based toggle buttons to switch between measurement modes
- **Automatic Boundary Detection**: Lines extend until a significant color change is detected
- **Visual Feedback**: Selected tool is highlighted with different colors

## Dependencies

- `gtk4` - GUI toolkit
- `gtk4-layer-shell` - Window layer management
- `image` - Image processing
- `glib` - GLib bindings
- `time` - Time handling
- `grim` - Required external tool for taking screenshots (on Wayland)

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (edition 2024)
- On Wayland: `grim` (screenshot utility)
- On X11: You may need to install additional Wayland compatibility libraries

## Installation

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
4. Press `Ctrl` to show/hide the control center with tool selection buttons
5. Select different measurement modes using the image-based toggle buttons:
   - Cross icon: Full crosshair mode (measures width × height)
   - Line icon: Horizontal line mode (measures width only)
   - Rotated line icon: Vertical line mode (measures height only)
6. The measurements will adjust based on the selected tool
7. Press `Escape` to exit the application and clean up temporary files

## How It Works

Rustyruler works by:

1. Capturing a screenshot of the current screen using the `grim` utility
2. Displaying the screenshot as a fullscreen overlay using GTK4 and layer-shell
3. Tracking mouse movements to position the crosshair
4. Analyzing the screenshot pixel by pixel to detect color changes
5. Calculating line limits based on the selected tool:
   - Cross tool: Calculates limits in all 4 directions
   - Horizontal line tool: Calculates only left/right limits, fixes top/bottom
   - Vertical line tool: Calculates only top/bottom limits, fixes left/right
6. Drawing the appropriate crosshair based on the selected tool
7. Displaying measurements appropriate to the selected tool

## Code Structure

- `main.rs`: Application entry point and GTK4 initialization
- `ui.rs`: GUI setup, drawing functions, event handling, and tool selection logic
- `screenshot.rs`: Screenshot capture, image loading, color analysis functions, and tool-specific calculations
