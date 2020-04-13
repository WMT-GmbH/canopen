pub trait Network {
    fn send_message(&self, can_id: u32, data: [u8; 8]);
}
