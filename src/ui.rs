use crate::screenshot;
use glib::Propagation;
use gtk4::{
    Application, ApplicationWindow, DrawingArea, EventControllerKey, EventControllerMotion, cairo,
    gdk::Key, gdk_pixbuf::Pixbuf, prelude::*,
};
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Clone)]
struct CrosshairData {
    x: u32,
    y: u32,
    top_limit: u32,
    bottom_limit: u32,
    left_limit: u32,
    right_limit: u32,
    initialized: bool,
}

pub fn build_ui(app: &Application) {
    // Capture the original screenshot for the background
    let original_screenshot_path: PathBuf = match screenshot::capture_original_screenshot() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("Error capturing original screenshot: {:?}", err);
            std::process::exit(1);
        }
    };

    // Create and configure the main application window
    let window = create_and_configure_window(app);

    // Keep a reference to the screenshot path for cleanup
    let screenshot_path_for_cleanup = original_screenshot_path.clone();

    // Load image data
    let (rgb_image, pixbuf) = load_image_data(&original_screenshot_path);

    let img_width = pixbuf.width() as u32;
    let img_height = pixbuf.height() as u32;

    // Initialize crosshair data structure
    let crosshair_data = Rc::new(RefCell::new(CrosshairData {
        x: 0,
        y: 0,
        top_limit: 0,
        bottom_limit: 0,
        left_limit: 0,
        right_limit: 0,
        initialized: false,
    }));

    // Store scale and offset information for coordinate transformations (shared between drawing and mouse events)
    let scale_and_offset = Rc::new(RefCell::new((1.0_f64, 0.0_f64, 0.0_f64)));

    // Create drawing area and set up drawing function
    let drawing_area = create_drawing_area(
        &pixbuf,
        img_width,
        img_height,
        crosshair_data.clone(),
        scale_and_offset.clone(),
    );

    // Set up keyboard and mouse event handling
    setup_event_handlers(
        &window,
        &drawing_area,
        crosshair_data.clone(),
        rgb_image.clone(),
        original_screenshot_path.clone(),
        scale_and_offset,
    );

    // Set up cleanup when window closes
    setup_cleanup(&window, screenshot_path_for_cleanup);

    // Present the window
    window.set_child(Some(&drawing_area));
    window.grab_focus();
    window.fullscreen();
    window.present();
}

/// Creates and configures the main application window
fn create_and_configure_window(app: &Application) -> ApplicationWindow {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Minha Layer Fullscreen")
        .can_focus(true)
        .build();

    // Configure the window as a layer shell overlay
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::Exclusive);
    window.set_anchor(gtk4_layer_shell::Edge::Left, true);
    window.set_anchor(gtk4_layer_shell::Edge::Right, true);
    window.set_anchor(gtk4_layer_shell::Edge::Top, true);
    window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
    window.set_exclusive_zone(-1);

    window
}

/// Loads image data from the captured screenshot
fn load_image_data(original_screenshot_path: &PathBuf) -> (Rc<image::RgbImage>, Pixbuf) {
    // Load the RGB image for color analysis
    let rgb_image = match screenshot::load_image(original_screenshot_path) {
        Ok(img) => Rc::new(img),
        Err(err) => {
            eprintln!("Error loading RGB image: {:?}", err);
            std::process::exit(1);
        }
    };

    // Load the image as Pixbuf for drawing
    let pixbuf = match Pixbuf::from_file(original_screenshot_path) {
        Ok(pb) => pb,
        Err(err) => {
            eprintln!("Error loading pixbuf: {:?}", err);
            std::process::exit(1);
        }
    };

    (rgb_image, pixbuf)
}

/// Draws the crosshair lines at the current position
fn draw_crosshair(cr: &cairo::Context, data: &CrosshairData, scale: f64) {
    // Set crosshair color to red and line width
    cr.set_source_rgb(1.0, 0.0, 0.0);
    cr.set_line_width(1.0 / scale);

    let lower_x = if data.x < 4 {
        0
    } else {
        data.x - 4
    };
    let upper_x = data.x + 4;

    // Draw vertical line above the crosshair center
    cr.move_to(data.x as f64, data.y as f64);
    cr.line_to(data.x as f64, data.top_limit as f64);
    let _ = cr.stroke();
    cr.move_to((lower_x) as f64, data.top_limit as f64);
    cr.line_to((upper_x) as f64, data.top_limit as f64);
    let _ = cr.stroke();
    cr.move_to((lower_x) as f64, (data.top_limit - 1) as f64);
    cr.line_to((upper_x) as f64, (data.top_limit - 1) as f64);
    let _ = cr.stroke();

    // Draw vertical line below the crosshair center
    cr.move_to(data.x as f64, data.y as f64);
    cr.line_to(data.x as f64, data.bottom_limit as f64);
    let _ = cr.stroke();
    cr.move_to((lower_x) as f64, data.bottom_limit as f64);
    cr.line_to((upper_x) as f64, data.bottom_limit as f64);
    let _ = cr.stroke();
    cr.move_to((lower_x) as f64, (data.bottom_limit - 1) as f64);
    cr.line_to((upper_x) as f64, (data.bottom_limit - 1) as f64);
    let _ = cr.stroke();

    let lower_y = if data.y < 4 {
        0
    } else {
        data.y - 4
    };
    let upper_y = data.y + 4;

    // Draw horizontal line to the left of crosshair center
    cr.move_to(data.x as f64, data.y as f64);
    cr.line_to(data.left_limit as f64, data.y as f64);
    let _ = cr.stroke();
    cr.move_to(data.left_limit as f64, (lower_y) as f64);
    cr.line_to(data.left_limit as f64, (upper_y) as f64);
    let _ = cr.stroke();
    cr.move_to((data.left_limit - 1) as f64, (lower_y) as f64);
    cr.line_to((data.left_limit - 1) as f64, (upper_y) as f64);
    let _ = cr.stroke();

    // Draw horizontal line to the right of crosshair center
    cr.move_to(data.x as f64, data.y as f64);
    cr.line_to(data.right_limit as f64, data.y as f64);
    let _ = cr.stroke();
    cr.move_to(data.right_limit as f64, (lower_y) as f64);
    cr.line_to(data.right_limit as f64, (upper_y) as f64);
    let _ = cr.stroke();
    cr.move_to((data.right_limit - 1) as f64, (lower_y) as f64);
    cr.line_to((data.right_limit - 1) as f64, (upper_y) as f64);
    let _ = cr.stroke();

    // Draw center point of the crosshair
    cr.arc(
        data.x as f64,
        data.y as f64,
        3.0 / scale,
        0.0,
        2.0 * std::f64::consts::PI,
    );
    cr.fill().unwrap();
}

/// Draws the tooltip with dimensions at the current position
fn draw_tooltip(
    cr: &cairo::Context,
    data: &CrosshairData,
    scale: f64,
    scale_and_offset: &RefCell<(f64, f64, f64)>,
    img_width: u32,
    img_height: u32,
) {
    // Draw text showing the dimensions of the current selection
    cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
    cr.set_font_size(20.0 / scale);

    let x_size = &data.right_limit - &data.left_limit + 1;
    let y_size = &data.bottom_limit - &data.top_limit + 1;
    let coords = format!("{} × {}", x_size, y_size);

    // Get text dimensions for background calculation
    let text_extents = cr.text_extents(&coords).unwrap();
    let text_width = text_extents.width();
    let text_height = text_extents.height();

    // Calculate initial position near crosshair center
    let text_offset_x = 25.0 / scale;
    let text_offset_y = 25.0 / scale;
    let mut x_pos = data.x as f64 + text_offset_x;
    let mut y_pos = data.y as f64 + text_offset_y + text_height; // Add text height to position below cursor

    // Calculate background dimensions
    let bg_padding = 8.0 / scale;
    let bg_width = text_width + 2.0 * bg_padding;
    let bg_height = text_height + 2.0 * bg_padding;
    let radius = 8.0 / scale; // Corner radius

    // Boundary checking to keep the box within visible area
    // Calculate the maximum visible coordinates (after scaling and offsetting)
    let (_, offset_x, offset_y) = *scale_and_offset.borrow();
    let visible_width = img_width as f64 * scale;
    let visible_height = img_height as f64 * scale;

    // Adjust x position if box would extend beyond right edge
    if x_pos + text_width + bg_padding > offset_x + visible_width {
        // Position to the left of the cursor with space between crosshair and box
        let space_from_cursor = 15.0 / scale; // Space between cursor and box
        x_pos = (data.x as f64 - text_width - bg_padding - space_from_cursor)
            .max(offset_x + bg_padding); // Position to the left of cursor if needed
    }

    // Adjust y position if box would extend beyond bottom edge
    if y_pos > offset_y + visible_height - bg_padding {
        // Position above the cursor
        let space_from_cursor = 25.0 / scale; // Space between cursor and box
        y_pos = data.y as f64 - space_from_cursor; // Position above cursor
    }

    // Calculate final background position
    let bg_x = x_pos - bg_padding;
    let bg_y = y_pos - text_height - bg_padding;

    // Create rounded rectangle path for the text background
    cr.new_sub_path();
    cr.arc(
        bg_x + radius,
        bg_y + radius,
        radius,
        180.0_f64.to_radians(),
        270.0_f64.to_radians(),
    );
    cr.line_to(bg_x + bg_width - radius, bg_y);
    cr.arc(
        bg_x + bg_width - radius,
        bg_y + radius,
        radius,
        270.0_f64.to_radians(),
        360.0_f64.to_radians(),
    );
    cr.line_to(bg_x + bg_width, bg_y + bg_height - radius);
    cr.arc(
        bg_x + bg_width - radius,
        bg_y + bg_height - radius,
        radius,
        0.0_f64.to_radians(),
        90.0_f64.to_radians(),
    );
    cr.line_to(bg_x + radius, bg_y + bg_height);
    cr.arc(
        bg_x + radius,
        bg_y + bg_height - radius,
        radius,
        90.0_f64.to_radians(),
        180.0_f64.to_radians(),
    );
    cr.close_path();

    // Draw background with semi-transparent black
    cr.set_source_rgba(0.1, 0.1, 0.1, 0.9);
    cr.fill_preserve().unwrap(); // fill and preserve the path for the border

    // Draw border
    cr.set_source_rgb(0.2, 0.2, 0.2);
    cr.set_line_width(2.0 / scale); // Border line width
    cr.stroke().unwrap();

    // Draw the text with improved color (white for better contrast)
    cr.set_source_rgb(1.0, 1.0, 1.0); // White color for better contrast
    cr.move_to(x_pos, y_pos);
    cr.show_text(&coords).unwrap();
}

/// Creates the drawing area with the drawing function
fn create_drawing_area(
    pixbuf: &Pixbuf,
    img_width: u32,
    img_height: u32,
    crosshair_data: Rc<RefCell<CrosshairData>>,
    scale_and_offset: Rc<RefCell<(f64, f64, f64)>>,
) -> DrawingArea {
    let drawing_area = DrawingArea::new();
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);

    // Set up the drawing function for the drawing area
    let pixbuf_clone = pixbuf.clone();
    let crosshair_data_clone = crosshair_data.clone();
    let scale_and_offset_clone = scale_and_offset.clone();

    drawing_area.set_draw_func(move |_, cr, width, height| {
        // Calculate scaling to fit image while maintaining aspect ratio
        let scale_x = width as f64 / img_width as f64;
        let scale_y = height as f64 / img_height as f64;
        let scale = scale_x.min(scale_y);

        // Calculate offsets to center the image
        let offset_x = (width as f64 - img_width as f64 * scale) / 2.0;
        let offset_y = (height as f64 - img_height as f64 * scale) / 2.0;

        // Store scale and offset for coordinate conversion
        *scale_and_offset_clone.borrow_mut() = (scale, offset_x, offset_y);

        // Apply transformations to the drawing context
        cr.save().unwrap();
        cr.translate(offset_x, offset_y);
        cr.scale(scale, scale);

        // Draw the background image
        cr.set_source_pixbuf(&pixbuf_clone, 0.0, 0.0);
        let _ = cr.paint();

        let data = crosshair_data_clone.borrow();

        // Draw the crosshair and coordinates if initialized
        if data.initialized {
            draw_crosshair(cr, &data, scale);
            draw_tooltip(
                cr,
                &data,
                scale,
                &scale_and_offset_clone,
                img_width,
                img_height,
            );
        }

        // Restore the drawing context
        cr.restore().unwrap();
    });

    drawing_area
}

/// Sets up keyboard and mouse event handlers
fn setup_event_handlers(
    window: &ApplicationWindow,
    drawing_area: &DrawingArea,
    crosshair_data: Rc<RefCell<CrosshairData>>,
    rgb_image: Rc<image::RgbImage>,
    screenshot_path: PathBuf,
    scale_and_offset: Rc<RefCell<(f64, f64, f64)>>,
) {
    // Set up keyboard event handling
    let window_clone_for_cleanup = window.clone();
    let screenshot_path_clone = screenshot_path.clone();
    let key_controller = EventControllerKey::new();
    key_controller.connect_key_pressed(move |_, key, _, _| match key {
        Key::Escape => {
            // Clean up the temporary screenshot file before closing
            if let Err(err) = screenshot::cleanup_screenshot(&screenshot_path_clone) {
                eprintln!("Error cleaning up screenshot: {:?}", err);
            }
            window_clone_for_cleanup.close();
            Propagation::Stop
        }
        _ => Propagation::Proceed,
    });
    window.add_controller(key_controller);

    // Set up mouse motion event handling
    setup_mouse_events(
        window,
        drawing_area,
        crosshair_data,
        rgb_image,
        scale_and_offset,
    );
}

/// Sets up mouse event handling
fn setup_mouse_events(
    window: &ApplicationWindow,
    drawing_area: &DrawingArea,
    crosshair_data: Rc<RefCell<CrosshairData>>,
    rgb_image: Rc<image::RgbImage>,
    scale_and_offset: Rc<RefCell<(f64, f64, f64)>>,
) {
    let drawing_area_clone = drawing_area.clone();
    let crosshair_data_clone = crosshair_data.clone();
    let rgb_image_clone = rgb_image.clone();

    let update_crosshair = move |x: f64, y: f64| {
        let (scale, offset_x, offset_y) = *scale_and_offset.borrow();

        let mouse_x = ((x - offset_x) / scale) as u32;
        let mouse_y = ((y - offset_y) / scale) as u32;

        if screenshot::validate_coordinates(&rgb_image_clone, mouse_x, mouse_y).is_err() {
            return;
        }

        let (top, bottom, left, right) =
            screenshot::calculate_line_limits(&rgb_image_clone, mouse_x, mouse_y);

        // Update crosshair data with new position and limits
        {
            let mut data = crosshair_data_clone.borrow_mut();
            data.x = mouse_x;
            data.y = mouse_y;
            data.top_limit = top + 1;
            data.bottom_limit = bottom + 1;
            data.left_limit = left + 1;
            data.right_limit = right + 1;
            data.initialized = true;
        }

        // Request redraw to show updated crosshair
        drawing_area_clone.queue_draw();
    };

    let update_crosshair_enter = update_crosshair.clone();
    let motion_controller = EventControllerMotion::new();

    // Handle mouse entering the window
    motion_controller.connect_enter(move |_, x, y| {
        update_crosshair_enter(x, y);
    });

    // Handle mouse movement within the window
    motion_controller.connect_motion(move |_, x, y| {
        update_crosshair(x, y);
    });

    window.add_controller(motion_controller);
}

/// Sets up cleanup function to run when window closes
fn setup_cleanup(window: &ApplicationWindow, screenshot_path: PathBuf) {
    // Set up cleanup when window is closed by connecting to the 'close-request' signal
    window.connect_close_request(move |_window| {
        // Clean up the temporary screenshot file
        if let Err(err) = screenshot::cleanup_screenshot(&screenshot_path) {
            eprintln!("Error cleaning up screenshot: {:?}", err);
        }
        gtk4::glib::Propagation::Proceed
    });
}
