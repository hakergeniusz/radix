//! Radix — an independent desktop calculator (io.github.hakergeniusz.Radix).

mod convert;
mod data;
mod dialogs;
mod entry;
mod fmt;
mod keypads;
mod popovers;
mod persist;
mod win;

use adw::prelude::*;
use gtk::glib;

const APP_ID: &str = "io.github.hakergeniusz.Radix";

fn main() -> glib::ExitCode {
    // Under headless Xvfb the llvmpipe path of the OLD GL renderer drew
    // nothing; NGL fixed it. GTK >= 4.20 renamed NGL to "gl" (and made it
    // the default), so pin the correct name per runtime version (an
    // externally-set GSK_RENDERER still wins).
    if std::env::var_os("GSK_RENDERER").is_none() {
        // Safe: this runs before any thread spawns.
        let renamed =
            gtk::major_version() > 4 || (gtk::major_version() == 4 && gtk::minor_version() >= 20);
        unsafe { std::env::set_var("GSK_RENDERER", if renamed { "gl" } else { "ngl" }) };
    }
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .build();
    app.connect_activate(|app| {
        win::RadixWindow::new(app).present();
    });
    app.run()
}
