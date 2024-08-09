pub mod app;

#[cfg(target_os = "android")]
mod android {
    pub use winit::platform::android::activity::AndroidApp;
    use rivik::Rivik;
    use crate::app;

    #[no_mangle]
    fn android_main(app: AndroidApp) {
        env_logger::init();
        Rivik::run(|rivik|{
            app::run(rivik);
        }, app);
    }
}