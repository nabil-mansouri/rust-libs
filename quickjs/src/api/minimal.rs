use flutter_rust_bridge::frb;

#[frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}
#[frb(external)]
pub enum DummyEnum {
    Test,
}