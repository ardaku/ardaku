use daku::api::{log, prompt};

type LolAllocator =
    lol_alloc::AssumeSingleThreaded<lol_alloc::FreeListAllocator>;

#[global_allocator]
static GA: LolAllocator = unsafe {
    lol_alloc::AssumeSingleThreaded::new(lol_alloc::FreeListAllocator::new())
};

#[no_mangle]
unsafe extern "C" fn run() {
    daku::run::start(main());
}

async fn main() {
    // Uncomment if you want panic info for debugging
    /*
    fn panic(panic_info: &std::panic::PanicInfo) {
        log::error!("Panic: {panic_info}");
        log::logger().flush();
        unreachable!();
    }

    std::panic::set_hook(Box::new(panic)); */

    log::set_max_level(log::LevelFilter::Debug);
    log::info!("Wait a minute...");
    log::info!("What is your name?");

    let mut name = String::new();
    prompt::read_line(&mut name).await;
    log::info!("Hello, {name}!");
}
