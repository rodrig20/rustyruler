use image::RgbImage;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use time::OffsetDateTime;

/// Captures the original screenshot once at the beginning of the application
/// Uses the 'grim' command-line tool to capture the screenshot with a timestamp-based filename
pub fn capture_original_screenshot() -> io::Result<PathBuf> {
    let now = OffsetDateTime::now_utc();
    let timestamp = now.unix_timestamp_nanos();

    let mut temp_path = std::env::temp_dir();
    temp_path.push(format!("rustyruler_original_{}.png", timestamp));

    let status = Command::new("grim").arg(&temp_path).status()?;

    if status.success() {
        Ok(temp_path)
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to execute grim",
        ))
    }
}

/// Deletes the specified screenshot file
/// Used to clean up temporary files when they are no longer needed
pub fn cleanup_screenshot(screenshot_path: &PathBuf) -> io::Result<()> {
    if screenshot_path.exists() {
        std::fs::remove_file(screenshot_path)
    } else {
        // File doesn't exist, return Ok to avoid error
        Ok(())
    }
}

/// Loads an image from file into memory as an RGB image
/// Used for color analysis and calculations
pub fn load_image(img_path: &PathBuf) -> io::Result<RgbImage> {
    let img = image::open(img_path).map_err(|e| {
        io::Error::new(io::ErrorKind::Other, format!("Failed to open image: {}", e))
    })?;

    Ok(img.to_rgb8())
}

/// Validates that the given coordinates are within the image bounds
/// Returns dimensions if coordinates are valid, error otherwise
pub fn validate_coordinates(img: &RgbImage, x: u32, y: u32) -> Result<(u32, u32), ()> {
    let (width, height) = img.dimensions();

    if x >= width || y >= height {
        return Err(());
    }

    Ok((width, height))
}

/// Calculates the line limits for the crosshair based on color changes
/// Extends lines in all 4 directions until a significant color change is detected
pub fn calculate_line_limits(
    img: &RgbImage,
    x: u32,
    y: u32,
    active_tool: i32,
) -> (u32, u32, u32, u32) {
    let (width, height) = img.dimensions();

    let mut top_limit: u32 = y;
    let mut bottom_limit: u32 = y;
    let mut left_limit: u32 = x;
    let mut right_limit: u32 = x;

    if active_tool == 0 || active_tool == 2 {
        top_limit = calculate_limit(img, x, y, 0, true);
        bottom_limit = calculate_limit(img, x, y, height - 1, true);
    }
    if active_tool == 0 || active_tool == 1 {
        left_limit = calculate_limit(img, x, y, 0, false);
        right_limit = calculate_limit(img, x, y, width - 1, false);
    }

    (top_limit, bottom_limit, left_limit, right_limit)
}

/// Helper function to calculate a single line limit in a specific direction
/// Used for finding boundaries where color changes significantly
fn calculate_limit(img: &RgbImage, x: u32, y: u32, end: u32, vertical: bool) -> u32 {
    let start = if vertical { y } else { x };
    let fixed = if vertical { x } else { y };

    let mut last_pixel = if vertical {
        *img.get_pixel(fixed, start)
    } else {
        *img.get_pixel(start, fixed)
    };

    let iter: Box<dyn Iterator<Item = u32>> = if start > end {
        Box::new((end..=start).rev())
    } else {
        Box::new(start..=end)
    };

    for pos in iter {
        let current_pixel = if vertical {
            img.get_pixel(fixed, pos)
        } else {
            img.get_pixel(pos, fixed)
        };

        let diff_r = (i16::from(current_pixel[0]) - i16::from(last_pixel[0])).abs() as u32;
        let diff_g = (i16::from(current_pixel[1]) - i16::from(last_pixel[1])).abs() as u32;
        let diff_b = (i16::from(current_pixel[2]) - i16::from(last_pixel[2])).abs() as u32;
        let magnitude = ((diff_r * diff_r + diff_g * diff_g + diff_b * diff_b) as f32).sqrt();

        if magnitude > 20.0 {
            return pos;
        }

        last_pixel = *current_pixel;
    }

    end
}
