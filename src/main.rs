use log::info;

fn main() {
    env_logger::init();
    info!("vim_rust: 2 + 3 = {}", vim_channel::add(2, 3));
}
