use super::*;

#[derive(Debug)]
pub struct SDOServer {
    id: u8,
    tx_address: u32,
}

impl SDOServer {
    pub fn new(id: u8, tx_address: u32) -> SDOServer {
        SDOServer { id, tx_address }
    }

    pub fn successful_download_acknowledgment_frame(
        self: &Self,
        index: u16,
        subindex: u8,
    ) -> CANOpenFrameResult {
        successful_download_acknowledgment_frame(self.id, self.tx_address, index, subindex)
    }

    pub fn sdo_abort_frame(
        self: &Self,
        index: u16,
        subindex: u8,
        abort_code: u32,
    ) -> CANOpenFrameResult {
        sdo_abort_frame(self.id, self.tx_address, index, subindex, abort_code)
    }

    pub fn download_1_byte_frame(
        self: &Self,
        index: u16,
        subindex: u8,
        data: u8,
    ) -> CANOpenFrameResult {
        download_1_byte_frame(self.id, self.tx_address, index, subindex, data)
    }

    pub fn download_2_bytes_frame(
        self: &Self,
        index: u16,
        subindex: u8,
        data: [u8; 2],
    ) -> CANOpenFrameResult {
        download_2_bytes_frame(self.id, self.tx_address, index, subindex, data)
    }

    pub fn download_3_bytes_frame(
        self: &Self,
        index: u16,
        subindex: u8,
        data: [u8; 3],
    ) -> CANOpenFrameResult {
        download_3_bytes_frame(self.id, self.tx_address, index, subindex, data)
    }

    pub fn download_4_bytes_frame(
        self: &Self,
        index: u16,
        subindex: u8,
        data: [u8; 4],
    ) -> CANOpenFrameResult {
        download_4_bytes_frame(self.id, self.tx_address, index, subindex, data)
    }
}