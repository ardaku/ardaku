use daku::api::{self, prompt};
use log::Level;

#[global_allocator]
static _GA: lol_alloc::FreeListAllocator = lol_alloc::FreeListAllocator::new();

async fn main() {
    api::log::init(Level::Debug);
    log::info!("Wait a minute...");
    log::info!("What is your name?");

    let mut name = String::new();
    prompt::read_line(&mut name).await;
    log::info!("Hello, {name}!");
}

#[no_mangle]
extern "C" fn run() {
    // Uncomment if you want panic info for debugging
    /*
    fn panic(panic_info: &std::panic::PanicInfo) {
        log::error!("Panic: {panic_info}");
        log::logger().flush();
        unreachable!();
    }

    std::panic::set_hook(Box::new(panic));
    */

    daku::run::block_on(main());
}
