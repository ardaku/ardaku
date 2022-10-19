use daku::{api::{self, prompt}};
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
    daku::run::block_on(main());
}
