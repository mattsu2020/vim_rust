use gtk::prelude::*;

#[no_mangle]
pub extern "C" fn rs_gui_run() {
    gtk::init().expect("Failed to initialize GTK");

    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Vim Rust GTK");
    window.set_default_size(800, 600);
    window.connect_destroy(|_| {
        gtk::main_quit();
    });
    window.show_all();

    gtk::main();
}
