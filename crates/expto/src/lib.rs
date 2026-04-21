pub mod rdmp;
pub mod ip;
pub mod api;
pub mod prelude;
pub mod config;

/// 初始化日志系统
pub fn init_log() {
    #[cfg(debug_assertions)]
    {
        use std::sync::Once;
        static INIT: Once = Once::new();

        INIT.call_once(|| {
            env_logger::init();
            log::info!("expto 日志系统已初始化");
        });
    }
}
