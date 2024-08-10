pub mod app;

#[cfg(target_os = "android")]
mod android {
    use crate::app;
    use log::LevelFilter;
    use rivik::Rivik;
    pub use winit::platform::android::activity::AndroidApp;

    #[no_mangle]
    fn android_main(app: AndroidApp) {
        android_logger::init_once(
            android_logger::Config::default().with_max_level(LevelFilter::Warn),
        );
        Rivik::run(
            |rivik| {
                app::run(rivik);
            },
            app,
        );
    }
}
