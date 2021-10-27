use embedded_can::{Frame, Id};

pub struct CanOpenFrame {
    id: Id,
    data: [u8; 8],
    dlc: u8,
    is_remote: bool,
}

impl Frame for CanOpenFrame {
    fn new(id: impl Into<Id>, data: &[u8]) -> Result<Self, ()> {
        if data.len() > 8 {
            return Err(());
        }
        let mut frame_data = [0; 8];
        frame_data[0..data.len()].copy_from_slice(data);
        Ok(CanOpenFrame {
            id: id.into(),
            data: frame_data,
            dlc: data.len() as u8,
            is_remote: false,
        })
    }

    fn new_remote(id: impl Into<Id>, dlc: usize) -> Result<Self, ()> {
        if dlc > 8 {
            return Err(());
        }
        Ok(CanOpenFrame {
            id: id.into(),
            data: [0; 8],
            dlc: dlc as u8,
            is_remote: false,
        })
    }

    fn is_extended(&self) -> bool {
        matches!(self.id, Id::Extended(_))
    }

    fn is_remote_frame(&self) -> bool {
        self.is_remote
    }

    fn id(&self) -> Id {
        self.id
    }

    fn dlc(&self) -> usize {
        self.dlc as usize
    }

    fn data(&self) -> &[u8] {
        &self.data[0..self.dlc as usize]
    }
}
