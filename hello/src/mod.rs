use daku::api::{self, prompt};
use log::Level;

async fn main() {
    api::log::init(Level::Debug).await;
    log::info!("Wait a minute...");
    log::info!("What is your name?");

    let mut name = String::new();
    prompt::read_line(&mut name).await;
    log::info!("Hello, {name}!");
}

#[no_mangle]
extern "C" fn run() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |p| {
        hook(p);
        log::error!("Panic: {:?}", p.to_string());
        log::logger().flush();
        unreachable!();
    }));

    daku::run::block_on(main());
}
