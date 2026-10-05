mod api;
mod app;

use leptos::mount::mount_to_body;

fn main() {
    // Mostra panics do Rust no console do navegador (F12) em vez de falhar em silêncio.
    console_error_panic_hook::set_once();
    mount_to_body(app::App);
}
