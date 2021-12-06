use embedded_can::StandardId;

#[derive(Default)]
pub struct Nmt<'a> {
    callback: Option<&'a mut dyn NmtCallback>,
}

impl<'a> Nmt<'a> {
    pub const NMT_REQUEST_ID: StandardId = StandardId::ZERO;

    pub fn add_callback(&mut self, callback: &'a mut dyn NmtCallback) {
        self.callback = Some(callback);
    }

    pub fn on_request<F: embedded_can::Frame>(&mut self, command_code: u8) -> Option<F> {
        if let Some(callback) = self.callback.as_mut() {
            if command_code == 129 {
                callback.on_reset_request()
            }
        }
        None
    }
}

pub trait NmtCallback {
    fn on_reset_request(&mut self);
}
