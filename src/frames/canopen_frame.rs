use failure::Error;

#[derive(Debug, Fail)]
pub enum CANOpenFrameError {
    #[fail(display = "the COB-ID of this frame is invalid ({})", cob_id)]
    InvalidCOBID { cob_id: u32 },
    #[fail(
        display = "data length should not exceed 8 bytes ({} > 8)",
        length
    )]
    InvalidDataLength { length: usize },
}

#[derive(Debug, PartialEq)]
pub struct CANOpenFrame {
    pub cob_id: u32,
    pub length: u8,
    pub data: [u8; 8],
    pub is_rtr: bool,
}

impl std::fmt::Display for CANOpenFrame {
    fn fmt(
        self: &Self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:03X} [{}]\t", self.cob_id, self.length)?;

        for byte in self.data.iter() {
            write!(f, "{:02X} ", byte);
        }

        Ok(())
    }
}

pub type CANOpenFrameResult = Result<CANOpenFrame, Error>;

impl CANOpenFrame {
    pub fn new(cob_id: u32, data: &[u8]) -> CANOpenFrameResult {
        CANOpenFrame::new_with_rtr(cob_id, data, false)
    }

    pub fn new_rtr(cob_id: u32, data: &[u8]) -> CANOpenFrameResult {
        CANOpenFrame::new_with_rtr(cob_id, data, true)
    }

    pub fn new_with_rtr(cob_id: u32, data: &[u8], is_rtr: bool) -> CANOpenFrameResult {
        if cob_id > 0x77F {
            return Err(CANOpenFrameError::InvalidCOBID { cob_id }.into());
        }
        if data.len() > 8 {
            return Err(CANOpenFrameError::InvalidDataLength { length: data.len() }.into());
        }

        let mut frame = CANOpenFrame {
            cob_id,
            length: data.len() as u8,
            data: [0; 8],
            is_rtr,
        };

        frame.data[..data.len()].clone_from_slice(&data[..]);

        Ok(frame)
    }

    pub fn cob_id(self: &Self) -> u32 {
        self.cob_id
    }

    pub fn length(self: &Self) -> u8 {
        self.length
    }

    pub fn data(self: &Self) -> &[u8; 8] {
        &self.data
    }

    pub fn is_rtr(self: &Self) -> bool {
        self.is_rtr
    }
}