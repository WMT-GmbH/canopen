use super::*;
use crate::split::Split;

#[derive(Debug, Copy, Clone)]
pub enum State {
    BootUp,
    Operational,
    Stopped,
    PreOperational,
    UnknownState,
}

#[derive(Debug, Copy, Clone)]
pub enum Mode {
    Operational,
    Stop,
    PreOperational,
    ResetApplication,
    ResetCommunication,
}

pub fn sync_frame() -> CANOpenFrameResult {
    CANOpenFrame::new(0x080u32, &[])
}

pub fn set_mode_frame(id: u8, mode: Mode) -> CANOpenFrameResult {
    let mode_value = match mode {
        Mode::Operational => 1,
        Mode::Stop => 2,
        Mode::PreOperational => 80,
        Mode::ResetApplication => 81,
        Mode::ResetCommunication => 82,
    };

    CANOpenFrame::new(0x000u32, &[mode_value, id])
}

pub fn set_all_mode_frame(mode: Mode) -> CANOpenFrameResult {
    set_mode_frame(0u8, mode)
}

pub fn request_mode_frame(id: u8) -> CANOpenFrameResult {
    CANOpenFrame::new_with_rtr(0x700u32 + u32::from(id), &[], true)
}

pub fn guarding_frame(id: u8, state: State, toggle: bool) -> CANOpenFrameResult {
    let mut state_value = match state {
        State::BootUp => 0x00,
        State::Operational => 0x05,
        State::Stopped => 0x04,
        State::PreOperational => 0x7F,
        _ => panic!("will not send unknown state"),
    };

    if toggle {
        state_value |= 0x80;
    }

    CANOpenFrame::new(0x700u32 + u32::from(id), &[state_value])
}

pub fn heartbeat_frame(id: u8, state: State) -> CANOpenFrameResult {
    guarding_frame(id, state, false)
}

pub fn download_1_byte_frame(
    id: u8,
    rx_address: u32,
    index: u16,
    subindex: u8,
    data: u8,
) -> CANOpenFrameResult {
    CANOpenFrame::new(
        rx_address + u32::from(id),
        &[
            0x2F,
            index.lo(),
            index.hi(),
            subindex,
            data,
            0x00,
            0x00,
            0x00,
        ],
    )
}

pub fn download_2_bytes_frame(
    id: u8,
    rx_address: u32,
    index: u16,
    subindex: u8,
    data: [u8; 2],
) -> CANOpenFrameResult {
    CANOpenFrame::new(
        rx_address + u32::from(id),
        &[
            0x2B,
            index.lo(),
            index.hi(),
            subindex,
            data[0],
            data[1],
            0x00,
            0x00,
        ],
    )
}

pub fn download_3_bytes_frame(
    id: u8,
    rx_address: u32,
    index: u16,
    subindex: u8,
    data: [u8; 3],
) -> CANOpenFrameResult {
    CANOpenFrame::new(
        rx_address + u32::from(id),
        &[
            0x27,
            index.lo(),
            index.hi(),
            subindex,
            data[0],
            data[1],
            data[2],
            0x00,
        ],
    )
}

pub fn download_4_bytes_frame(
    id: u8,
    rx_address: u32,
    index: u16,
    subindex: u8,
    data: [u8; 4],
) -> CANOpenFrameResult {
    CANOpenFrame::new(
        rx_address + u32::from(id),
        &[
            0x23,
            index.lo(),
            index.hi(),
            subindex,
            data[0],
            data[1],
            data[2],
            data[3],
        ],
    )
}

pub fn successful_download_acknowledgment_frame(
    id: u8,
    tx_address: u32,
    index: u16,
    subindex: u8,
) -> CANOpenFrameResult {
    CANOpenFrame::new(
        tx_address + u32::from(id),
        &[
            0x60,
            index.lo(),
            index.hi(),
            subindex,
            0x00,
            0x00,
            0x00,
            0x00,
        ],
    )
}

pub fn sdo_abort_frame(
    id: u8,
    tx_address: u32,
    index: u16,
    subindex: u8,
    abort_code: u32,
) -> CANOpenFrameResult {
    CANOpenFrame::new(
        tx_address + u32::from(id),
        &[
            0x60,
            index.lo(),
            index.hi(),
            subindex,
            abort_code.lo().lo(),
            abort_code.lo().hi(),
            abort_code.hi().lo(),
            abort_code.hi().hi(),
        ],
    )
}

pub fn upload_request_frame(
    id: u8,
    rx_address: u32,
    index: u16,
    subindex: u8,
) -> CANOpenFrameResult {
    CANOpenFrame::new(
        rx_address + u32::from(id),
        &[
            0x40,
            index.lo(),
            index.hi(),
            subindex,
            0x00,
            0x00,
            0x00,
            0x00,
        ],
    )
}

pub fn upload_1_byte_frame(
    id: u8,
    tx_address: u32,
    index: u16,
    subindex: u8,
    data: u8,
) -> CANOpenFrameResult {
    CANOpenFrame::new(
        tx_address + u32::from(id),
        &[
            0x4F,
            index.lo(),
            index.hi(),
            subindex,
            data,
            0x00,
            0x00,
            0x00,
        ],
    )
}

pub fn upload_2_bytes_frame(
    id: u8,
    tx_address: u32,
    index: u16,
    subindex: u8,
    data: [u8; 2],
) -> CANOpenFrameResult {
    CANOpenFrame::new(
        tx_address + u32::from(id),
        &[
            0x4B,
            index.lo(),
            index.hi(),
            subindex,
            data[0],
            data[1],
            0x00,
            0x00,
        ],
    )
}

pub fn upload_3_bytes_frame(
    id: u8,
    tx_address: u32,
    index: u16,
    subindex: u8,
    data: [u8; 3],
) -> CANOpenFrameResult {
    CANOpenFrame::new(
        tx_address + u32::from(id),
        &[
            0x47,
            index.lo(),
            index.hi(),
            subindex,
            data[0],
            data[1],
            data[2],
            0x00,
        ],
    )
}

pub fn upload_4_bytes_frame(
    id: u8,
    tx_address: u32,
    index: u16,
    subindex: u8,
    data: [u8; 4],
) -> CANOpenFrameResult {
    CANOpenFrame::new(
        tx_address + u32::from(id),
        &[
            0x43,
            index.lo(),
            index.hi(),
            subindex,
            data[0],
            data[1],
            data[2],
            data[3],
        ],
    )
}

pub fn emergency_frame(
    id: u8,
    error_code: u16,
    error_register: u8,
    data: [u8; 5],
) -> CANOpenFrameResult {
    CANOpenFrame::new(
        0x80u32 + u32::from(id),
        &[
            error_code.lo(),
            error_code.hi(),
            error_register,
            data[0],
            data[1],
            data[2],
            data[3],
            data[4],
        ],
    )
}

pub fn get_mode(message: &CANOpenFrame) -> State {
    match message.data()[0] & 0x80 {
        0x04 => State::Stopped,
        0x05 => State::Operational,
        0x7F => State::PreOperational,
        _ => State::UnknownState,
    }
}