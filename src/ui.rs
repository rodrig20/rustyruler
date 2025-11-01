use crate::screenshot;
use glib::Propagation;
use gtk4::{
    Application, ApplicationWindow, Box, CssProvider, DrawingArea, EventControllerKey,
    EventControllerMotion, Overlay, cairo,
    gdk::{Display, Key},
    gdk_pixbuf::Pixbuf,
    prelude::*,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
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
    magnitude_threshold: f32,
}

pub fn build_ui(app: &Application) {
    let original_screenshot_path: PathBuf = match screenshot::capture_original_screenshot() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("Error capturing original screenshot: {:?}", err);
            std::process::exit(1);
        }
    };

    let window = create_and_configure_window(app);

    let provider = CssProvider::new();
    provider.load_from_path("assets/style.css");
    gtk4::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let screenshot_path_for_cleanup = original_screenshot_path.clone();
    let (rgb_image, pixbuf) = load_image_data(&original_screenshot_path);
    let img_width = pixbuf.width() as u32;
    let img_height = pixbuf.height() as u32;

    let crosshair_data = Rc::new(RefCell::new(CrosshairData {
        x: 0,
        y: 0,
        top_limit: 0,
        bottom_limit: 0,
        left_limit: 0,
        right_limit: 0,
        initialized: false,
        magnitude_threshold: 20.0,
    }));

    let scale_and_offset = Rc::new(RefCell::new((1.0_f64, 0.0_f64, 0.0_f64)));
    // Track the currently selected tool: 0 = cross, 1 = line, 2 = rotated line
    let active_tool = Rc::new(RefCell::new(0));

    let drawing_area = create_drawing_area(
        &pixbuf,
        img_width,
        img_height,
        crosshair_data.clone(),
        scale_and_offset.clone(),
        active_tool.clone(),
    );

    let command_center = create_command_center(active_tool.clone());
    let overlay = Overlay::builder().child(&drawing_area).build();
    overlay.add_overlay(&command_center);
    command_center.set_visible(false);

    setup_event_handlers(
        &window,
        &drawing_area,
        crosshair_data.clone(),
        rgb_image.clone(),
        original_screenshot_path.clone(),
        scale_and_offset,
        &command_center,
        active_tool,
    );

    setup_cleanup(&window, screenshot_path_for_cleanup);

    window.set_child(Some(&overlay));
    window.grab_focus();
    window.fullscreen();
    window.present();
}

/// Draws a custom command center with straight top/bottom edges and inward-curved sides
fn draw_command_center(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    curve_depth: f64, // How much the sides curve inward
    scale: f64,
    fill_color: (f64, f64, f64, f64),
    border_color: (f64, f64, f64),
    border_width: f64,
) {
    // Create a shape with straight top/bottom and curved sides
    cr.new_sub_path();

    // Start at top-left
    cr.move_to(x, y);

    // Top edge - straight line
    cr.line_to(x + width, y);

    // Right edge with inward curve
    cr.curve_to(
        x + width - curve_depth * 1.8,
        y + height * 0.35,
        x + width * 0.95 - curve_depth * 0.2,
        y + height * 0.9,
        x + width * 0.9,
        y + height,
    );

    // Bottom edge - straight line
    cr.line_to(x + width * 0.1, y + height);

    // Left edge with inward curve (to close the shape)
    cr.curve_to(
        x + width * 0.05 + curve_depth * 0.2,
        y + height * 0.9, // control point 1
        x + curve_depth * 1.8,
        y + height * 0.35, // control point 2
        x,
        y, // end point (back to start)
    );

    cr.close_path();

    // Fill with specified color
    cr.set_source_rgba(fill_color.0, fill_color.1, fill_color.2, fill_color.3);
    cr.fill_preserve().unwrap();

    // Draw border with specified color and width
    cr.set_source_rgb(border_color.0, border_color.1, border_color.2);
    cr.set_line_width(border_width / scale);
    cr.stroke().unwrap();
}

fn create_command_center(active_tool: Rc<RefCell<i32>>) -> Box {
    // Create a container box for the command center
    let command_center_box = Box::builder()
        .css_classes(vec!["command-center-outer"])
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Start)
        .margin_top(0) // Remove top margin to eliminate gap
        .margin_bottom(5) // Reduced margin
        .width_request(200) // Better width for square buttons
        .height_request(60) // Adjusted for square buttons
        .build();

    // Create a drawing area to draw the rounded background
    let background_drawing_area = DrawingArea::new();
    background_drawing_area.set_hexpand(true);
    background_drawing_area.set_vexpand(true);
    background_drawing_area.set_size_request(200, 60); // Adjusted size

    // Create image toggle buttons
    let image1 = gtk4::Image::from_file("assets/cross.png");
    image1.set_pixel_size(16); // Reduced image size to fit with less padding
    let button1 = gtk4::ToggleButton::new();
    button1.set_child(Some(&image1));

    let image2 = gtk4::Image::from_file("assets/line.png");
    image2.set_pixel_size(16); // Reduced image size to fit with less padding
    let button2 = gtk4::ToggleButton::new();
    button2.set_child(Some(&image2));

    // For the third button, we'll create a 90-degree rotated version of the line.png
    // by using the image crate to rotate the image data and then create a GDK texture
    let rotated_texture = {
        // Load the original image using the image crate
        let original_img = image::open("assets/line.png").expect("Failed to load line.png");
        let rotated_img = original_img.rotate90(); // Rotate 90 degrees clockwise

        // Get RGBA data from the rotated image
        let rgba = rotated_img.to_rgba8();
        let width = rgba.width();
        let height = rgba.height();
        let raw_data = rgba.as_raw();

        // Create a GDK memory texture from the RGBA data
        gtk4::gdk::MemoryTexture::new(
            width as i32,
            height as i32,
            gtk4::gdk::MemoryFormat::R8g8b8a8,
            &glib::Bytes::from(raw_data),
            width as usize * 4,
        )
    };

    let image3 = gtk4::Image::from_paintable(Some(&rotated_texture));
    image3.set_pixel_size(16); // Reduced image size to fit with less padding
    let button3 = gtk4::ToggleButton::new();
    button3.set_child(Some(&image3));

    // Make buttons behave like radio buttons (only one selected at a time)
    // Connect signals to ensure only one button is active at a time and track the active tool
    button1.connect_toggled({
        let button2_clone = button2.clone();
        let button3_clone = button3.clone();
        let active_tool_clone = active_tool.clone();
        move |btn| {
            if btn.is_active() {
                button2_clone.set_active(false);
                button3_clone.set_active(false);
                *active_tool_clone.borrow_mut() = 0;
            }
        }
    });

    button2.connect_toggled({
        let button1_clone = button1.clone();
        let button3_clone = button3.clone();
        let active_tool_clone = active_tool.clone();
        move |btn| {
            if btn.is_active() {
                button1_clone.set_active(false);
                button3_clone.set_active(false);
                *active_tool_clone.borrow_mut() = 1;
            }
        }
    });

    button3.connect_toggled({
        let button1_clone = button1.clone();
        let button2_clone = button2.clone();
        let active_tool_clone = active_tool.clone();
        move |btn| {
            if btn.is_active() {
                button1_clone.set_active(false);
                button2_clone.set_active(false);
                *active_tool_clone.borrow_mut() = 2;
            }
        }
    });

    // Set the first button (cross tool) as active by default
    button1.set_active(true);

    // Apply basic styling to toggle buttons to make them look good over the rounded background
    button1.set_can_focus(true);
    button2.set_can_focus(true);
    button3.set_can_focus(true);

    // Create an overlay to position the buttons over the background
    let overlay = Overlay::new();
    overlay.set_child(Some(&background_drawing_area));

    // Position the buttons using fixed layout
    let button_container = Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .spacing(20) // Adjusted spacing for square buttons
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .build();

    button_container.append(&button1);
    button_container.append(&button2);
    button_container.append(&button3);

    overlay.add_overlay(&button_container);

    // Set up the drawing function for the background
    background_drawing_area.set_draw_func(move |_, cr, width, height| {
        // Draw the rounded command center background with rounded bottom
        draw_command_center(
            cr,
            0.0,                     // x position
            0.0,                     // y position
            width as f64,            // width
            height as f64,           // height
            10.0,                    // smaller curve depth for more subtle shape
            1.0,                     // scale (since this is already scaled by GTK)
            (0.08, 0.08, 0.08, 0.8), // fill color (more opaque black)
            (0.3, 0.3, 0.3),         // border color (light gray)
            2.0,                     // thinner border
        );
    });

    command_center_box.append(&overlay);

    command_center_box
}

fn create_and_configure_window(app: &Application) -> ApplicationWindow {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Minha Layer Fullscreen")
        .can_focus(true)
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::Exclusive);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);
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
/// Draws the crosshair lines at the current position based on the active tool
/// - Tool 0 (Cross): Draws full crosshair (both vertical and horizontal lines)
/// - Tool 1 (Horizontal line): Draws only horizontal line
/// - Tool 2 (Vertical line): Draws only vertical line
fn draw_crosshair(cr: &cairo::Context, data: &CrosshairData, scale: f64, active_tool: i32) {
    // Set crosshair color to red and line width

    cr.set_source_rgb(1.0, 0.0, 0.0);
    cr.set_line_width(1.0 / scale);

    if active_tool == 0 || active_tool == 2 {
        let lower_x = data.x.saturating_sub(4);
        let upper_x = data.x + 4;

        // Draw vertical line above the crosshair center
        cr.move_to(data.x as f64, data.y as f64);
        cr.line_to(data.x as f64, data.top_limit as f64);
        let _ = cr.stroke();
        cr.move_to((lower_x) as f64, data.top_limit as f64);
        cr.line_to((upper_x) as f64, data.top_limit as f64);
        let _ = cr.stroke();
        cr.move_to((lower_x) as f64, (data.top_limit + 1) as f64);
        cr.line_to((upper_x) as f64, (data.top_limit + 1) as f64);
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
    }
    if active_tool == 0 || active_tool == 1 {
        let lower_y = data.y.saturating_sub(4);
        let upper_y = data.y + 4;

        // Draw horizontal line to the left of crosshair center
        cr.move_to(data.x as f64, data.y as f64);
        cr.line_to(data.left_limit as f64, data.y as f64);
        let _ = cr.stroke();
        cr.move_to(data.left_limit as f64, (lower_y) as f64);
        cr.line_to(data.left_limit as f64, (upper_y) as f64);
        let _ = cr.stroke();
        cr.move_to((data.left_limit + 1) as f64, (lower_y) as f64);
        cr.line_to((data.left_limit + 1) as f64, (upper_y) as f64);
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
}

/// Draws the tooltip with dimensions at the current position based on the active tool
fn draw_tooltip(
    cr: &cairo::Context,
    data: &CrosshairData,
    scale: f64,
    scale_and_offset: &RefCell<(f64, f64, f64)>,
    img_width: u32,
    img_height: u32,
    active_tool: i32,
) {
    // Draw text showing the dimensions of the current selection
    cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
    cr.set_font_size(20.0 / scale);
    let coords;
    if active_tool == 0 {
        let x_size = data.right_limit - data.left_limit + 1;
        let y_size = data.bottom_limit - data.top_limit + 1;
        coords = format!("{x_size} × {y_size}");
    } else if active_tool == 1 {
        let x_size = data.right_limit - data.left_limit + 1;
        coords = format!("{x_size}");
    } else {
        let y_size = data.bottom_limit - data.top_limit + 1;
        coords = format!("{y_size}");
    }

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
    active_tool: Rc<RefCell<i32>>,
) -> DrawingArea {
    let drawing_area = DrawingArea::new();
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);

    // Set up the drawing function for the drawing area
    let pixbuf_clone = pixbuf.clone();
    let crosshair_data_clone = crosshair_data.clone();
    let scale_and_offset_clone = scale_and_offset.clone();
    let active_tool_clone = active_tool.clone();

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
            let current_tool = *active_tool_clone.borrow();
            draw_crosshair(cr, &data, scale, current_tool);
            draw_tooltip(
                cr,
                &data,
                scale,
                &scale_and_offset_clone,
                img_width,
                img_height,
                current_tool,
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
    command_center: &Box,
    active_tool: Rc<RefCell<i32>>,
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

    let command_center_clone_press = command_center.clone();
    let key_controller_press = EventControllerKey::new();
    key_controller_press.connect_key_pressed(move |_, key, _, _| {
        if key == Key::Control_L || key == Key::Control_R {
            command_center_clone_press.set_visible(true);
            return Propagation::Stop;
        }
        Propagation::Proceed
    });
    window.add_controller(key_controller_press);

    let command_center_clone_release = command_center.clone();
    let key_controller_release = EventControllerKey::new();
    key_controller_release.connect_key_released(move |_, key, _, _| {
        if key == Key::Control_L || key == Key::Control_R {
            command_center_clone_release.set_visible(false);
        }
    });
    window.add_controller(key_controller_release);

    // Set up mouse motion event handling
    setup_mouse_events(
        window,
        drawing_area,
        crosshair_data,
        rgb_image,
        scale_and_offset,
        active_tool,
    );
}

/// Sets up mouse event handling
fn setup_mouse_events(
    window: &ApplicationWindow,
    drawing_area: &DrawingArea,
    crosshair_data: Rc<RefCell<CrosshairData>>,
    rgb_image: Rc<image::RgbImage>,
    scale_and_offset: Rc<RefCell<(f64, f64, f64)>>,
    active_tool: Rc<RefCell<i32>>,
) {
    let drawing_area_clone = drawing_area.clone();
    let crosshair_data_clone = crosshair_data.clone();
    let rgb_image_clone = rgb_image.clone();
    let active_tool_clone = active_tool.clone();

    let update_crosshair = move |x: f64, y: f64| {
        let (scale, offset_x, offset_y) = *scale_and_offset.borrow();

        let mut mouse_x = ((x - offset_x) / scale) as u32;
        let mut mouse_y = ((y - offset_y) / scale) as u32;

        let (img_width, img_height) = rgb_image_clone.dimensions();

        // Clamp coordinates to be within image bounds
        if mouse_x >= img_width {
            mouse_x = img_width - 1
        }
        if mouse_y >= img_height {
            mouse_y = img_height - 1;
        }

        if screenshot::validate_coordinates(&rgb_image_clone, mouse_x, mouse_y).is_err() {
            return;
        }

        let current_tool = *active_tool_clone.borrow();
        let magnitude_threshold = crosshair_data_clone.borrow().magnitude_threshold;
        let (top, bottom, left, right) = screenshot::calculate_line_limits(
            &rgb_image_clone,
            mouse_x,
            mouse_y,
            current_tool,
            magnitude_threshold,
        );

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

    // Set up scroll event handling for magnitude adjustment
    let drawing_area_scroll = drawing_area.clone();
    let crosshair_data_scroll = crosshair_data.clone();
    let rgb_image_clone_for_scroll = rgb_image.clone();
    let active_tool_clone_for_scroll = active_tool.clone();
    let scroll_controller =
        gtk4::EventControllerScroll::new(gtk4::EventControllerScrollFlags::VERTICAL);
    scroll_controller.connect_scroll(move |_, _, y_scroll| {
        // Adjust magnitude threshold based on scroll direction
        let mut data = crosshair_data_scroll.borrow_mut();
        let mut new_magnitude = data.magnitude_threshold;

        // Scrolling down
        if y_scroll > 0.0 {
            let scale_factor = (new_magnitude / 20.0).max(0.5);
            new_magnitude = (new_magnitude * (1.0 + 0.05 * scale_factor)).min(255.0);
        // Scrolling up
        } else if y_scroll < 0.0 {
            let scale_factor = (new_magnitude / 20.0).max(0.5);
            new_magnitude = (new_magnitude / (1.0 + 0.05 * scale_factor)).max(1.0);
        }

        data.magnitude_threshold = new_magnitude;

        // Recalculate the limits with the new magnitude threshold using the current position
        if data.initialized {
            let current_tool = *active_tool_clone_for_scroll.borrow();
            let (top, bottom, left, right) = screenshot::calculate_line_limits(
                &rgb_image_clone_for_scroll,
                data.x,
                data.y,
                current_tool,
                new_magnitude,
            );

            // Update the limits with the new calculation
            data.top_limit = top + 1;
            data.bottom_limit = bottom + 1;
            data.left_limit = left + 1;
            data.right_limit = right + 1;
        }

        // Update the crosshair to reflect the new magnitude
        drawing_area_scroll.queue_draw();
        gtk4::glib::Propagation::Proceed
    });

    drawing_area.add_controller(scroll_controller);
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
