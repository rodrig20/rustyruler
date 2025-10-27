mod screenshot;
mod ui;

use gtk4::{Application, prelude::*};
use ui::build_ui;

const APP_ID: &str = "com.rodrig20.rustyruler";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}
