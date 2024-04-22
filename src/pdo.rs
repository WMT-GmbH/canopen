use core::num::{NonZeroU16, NonZeroU8};

use embedded_can::{ExtendedId, Id, StandardId};

use crate::objectdictionary::datalink::{DataLink, ReadData, WriteStream};
use crate::objectdictionary::ODError;
use crate::sdo::SDOAbortCode;
use crate::NodeId;
use crate::ObjectDictionary;

pub struct TPDO {
    pub com: PDOCommunicationParameter,
    pub map: TPDOMappingParameters,
}

impl TPDO {
    #[inline]
    pub fn new(com: PDOCommunicationParameter, map: TPDOMappingParameters) -> Self {
        TPDO { com, map }
    }

    pub fn create_frame<F: embedded_can::Frame, T, const N: usize>(
        &self,
        od: &mut ObjectDictionary<T, N>,
    ) -> Result<F, SDOAbortCode> {
        let mut buf = [0; 8];
        let mut frame_len = 0;
        for i in 0..self.map.num_mapped_variables as usize {
            if let Some(position) = self.map.map[i] {
                let data = od.get(position).read(0, 0)?; //TODO index, subindex
                let bytes = data.as_bytes();
                buf[frame_len..frame_len + bytes.len()].copy_from_slice(bytes);
                frame_len += bytes.len();
            }
        }

        Ok(F::new(self.com.cob_id().id, &buf[0..frame_len]).unwrap())
    }

    /*    /// index 0x1800h to 0x19FF
    pub fn cob_id_variable(&self, index: u16) -> Variable<'_> {
        Variable::new_datalink_ref(index, 1, self, None)
    }
    /// index 0x1800h to 0x19FF
    pub fn transmission_type_variable(&self, index: u16) -> Variable<'_> {
        Variable::new_datalink_ref(index, 2, self, None)
    }
    /// index 0x1800h to 0x19FF
    pub fn inhibit_time_variable(&self, index: u16) -> Variable<'_> {
        Variable::new_datalink_ref(index, 3, self, None)
    }
    /// index 0x1800h to 0x19FF
    pub fn event_timer_variable(&self, index: u16) -> Variable<'_> {
        Variable::new_datalink_ref(index, 5, self, None)
    }
    /// index 0x1800h to 0x19FF
    pub fn sync_start_value_variable(&self, index: u16) -> Variable<'_> {
        Variable::new_datalink_ref(index, 6, self, None)
    }*/
}

impl DataLink for TPDO {
    fn read(&self, index: u16, subindex: u8) -> Result<ReadData<'_>, ODError> {
        match index {
            0x1800..=0x19FF => match subindex {
                1 => Ok(self.com.cob_id.into()),
                2 => Ok(self.com.transmission_type.into()),
                3 => Ok(self.com.inhibit_time.into()),
                5 => Ok(self.com.event_timer.into()),
                6 => Ok(self.com.sync_start_value.into()),
                _ => unreachable!(),
            },
            0x1A00..=0x1BFF => match subindex {
                0 => Ok(self.map.num_mapped_variables.into()),
                n => Ok(self.map.get_map_data_packed(n).into()),
            },
            _ => unreachable!(),
        }
    }

    fn write(&mut self, write_stream: WriteStream<'_>) -> Result<(), ODError> {
        // if currently valid, the only allowed write is to the valid bit
        if self.com.cob_id().valid && (write_stream.index > 0x19FF || write_stream.subindex != 1) {
            return Err(ODError::DeviceStateError);
        }

        match write_stream.index {
            0x1800..=0x19FF => match write_stream.subindex {
                1 => {
                    let new_cob_id = u32::try_from(write_stream)?;
                    let new_cob_id =
                        (self.com.cob_id_update_func)(self.com.cob_id(), CobId::from(new_cob_id))?;

                    self.com.cob_id = new_cob_id.into();
                }
                2 => self.com.transmission_type = write_stream.try_into()?,
                3 => self.com.inhibit_time = write_stream.try_into()?,
                5 => self.com.event_timer = write_stream.try_into()?,
                6 => self.com.sync_start_value = write_stream.try_into()?,
                _ => unreachable!(),
            },
            0x1A00..=0x1BFF => {
                if write_stream.subindex == 0 {
                    self.map.num_mapped_variables = write_stream.try_into()?;
                    return Ok(());
                }
                if self.map.num_mapped_variables > 0 {
                    // num_mapped_objects needs to be set to 0 before updating mapping
                    return Err(ODError::DeviceStateError);
                }
                if let Ok(data) = write_stream.new_data.try_into() {
                    let data = <u32>::from_le_bytes(data);
                    let (index, subindex, num_bits) = unpack_variable_data(data);

                    return match self.od.get().find(index, subindex) {
                        Ok(variable) => {
                            // validate num_bits
                            if num_bits % 8 != 0
                                || NonZeroU8::new(num_bits / 8) != variable.pdo_size
                            {
                                return Err(ODError::ObjectCannotBeMapped);
                            }

                            self.map.map_variable(write_stream.subindex, variable)
                        }
                        Err(_) => return Err(ODError::ObjectDoesNotExist),
                    };
                }
            }
            _ => unreachable!(),
        }
        Ok(())
    }
}

#[derive(Copy, Clone)]
#[repr(u16)]
pub enum DefaultTPDO {
    TPDO1 = 0,
    TPDO2 = 1,
    TPDO3 = 2,
    TPDO4 = 3,
}

impl DefaultTPDO {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        self,
        node_id: NodeId,
        cob_id_update_func: fn(CobId, CobId) -> Result<CobId, InvalidCobId>,
    ) -> TPDO {
        TPDO::new(
            PDOCommunicationParameter::new(self.cob_id(node_id, false, false), cob_id_update_func),
            TPDOMappingParameters::default(),
        )
    }

    pub fn cob_id(self, node_id: NodeId, valid: bool, rtr: bool) -> CobId {
        // SAFETY: Maximum StandardId is 0x7FF, maximum self is 3, maximum node_id is 0x7F
        let id = unsafe {
            Id::Standard(StandardId::new_unchecked(
                0x180 + 0x100 * self as u16 + node_id.raw() as u16,
            ))
        };
        CobId { valid, rtr, id }
    }
}

pub struct CobId {
    pub valid: bool,
    /// Only meaningful for TPDO
    pub rtr: bool,
    pub id: Id,
}

impl From<CobId> for u32 {
    fn from(cob_id: CobId) -> Self {
        match cob_id.id {
            Id::Standard(id) => {
                ((cob_id.valid as u32) << 31) + ((cob_id.rtr as u32) << 30) + id.as_raw() as u32
            }
            Id::Extended(id) => {
                ((cob_id.valid as u32) << 31)
                    + ((cob_id.rtr as u32) << 30)
                    + (1 << 29)
                    + id.as_raw()
            }
        }
    }
}
impl From<u32> for CobId {
    fn from(val: u32) -> Self {
        // SAFETY: bitmasks ensure id invariant
        let id = unsafe {
            if val & (1 << 29) > 0 {
                Id::Extended(ExtendedId::new_unchecked(val & ExtendedId::MAX.as_raw()))
            } else {
                Id::Standard(StandardId::new_unchecked(
                    val as u16 & StandardId::MAX.as_raw(),
                ))
            }
        };
        CobId {
            valid: val & (1 << 31) > 0,
            rtr: val & (1 << 30) > 0,
            id,
        }
    }
}

pub struct InvalidCobId;

impl From<InvalidCobId> for ODError {
    fn from(_: InvalidCobId) -> Self {
        ODError::InvalidValue
    }
}

#[derive(Copy, Clone)]
pub enum TPDOTransmissionType {
    SynchronousAcyclic,
    SynchronousEveryNSync(u8), // 1-240
    SynchronousRtrOnly,
    EventDrivenRtrOnly,
    EventDrivenManufacturerSpecific,
    EventDrivenProfileSpecific,
}

impl TPDOTransmissionType {
    pub const fn as_u8(self) -> u8 {
        match self {
            TPDOTransmissionType::SynchronousAcyclic => 0,
            TPDOTransmissionType::SynchronousEveryNSync(n) => n,
            TPDOTransmissionType::SynchronousRtrOnly => 0xFC,
            TPDOTransmissionType::EventDrivenRtrOnly => 0xFD,
            TPDOTransmissionType::EventDrivenManufacturerSpecific => 0xFE,
            TPDOTransmissionType::EventDrivenProfileSpecific => 0xFF,
        }
    }
}

pub struct PDOCommunicationParameter {
    cob_id: u32,
    transmission_type: u8,
    inhibit_time: u16,
    event_timer: u16,
    sync_start_value: u8,
    cob_id_update_func: fn(CobId, CobId) -> Result<CobId, InvalidCobId>,
}

impl PDOCommunicationParameter {
    pub fn new(
        cob_id: CobId,
        cob_id_update_func: fn(CobId, CobId) -> Result<CobId, InvalidCobId>,
    ) -> Self {
        PDOCommunicationParameter {
            cob_id: cob_id.into(),
            transmission_type: 0,
            inhibit_time: 0,
            event_timer: 0,
            sync_start_value: 0,
            cob_id_update_func,
        }
    }

    pub fn cob_id(&self) -> CobId {
        self.cob_id.into()
    }

    pub fn inhibit_time(&self) -> InhibitTime {
        self.inhibit_time.into()
    }
}

#[derive(Default)]
pub struct TPDOMappingParameters {
    /// The number of valid object entries within the mapping record.
    /// The number of valid object entries shall be the number of the application objects
    /// that shall be transmitted with the corresponding TPDO.
    num_mapped_variables: u8,
    map: [Option<usize>; 8],
}

impl TPDOMappingParameters {
    // slot 1-8
    pub fn map_variable(&mut self, slot: u8, od_position: usize) -> Result<(), ODError> {
        match variable.pdo_size {
            Some(n) if n.get() <= 8 => {
                // TODO check sizes, slot validity
                self.map[slot as usize] = Some(od_position);
                Ok(())
            }
            _ => Err(ODError::ObjectCannotBeMapped),
        }
    }

    #[inline]
    pub fn get_map_data_packed(&self, num: u8) -> u32 {
        match self.map[num as usize - 1] {
            Some(variable) => pack_variable_data(
                variable.index,
                variable.subindex,
                variable.pdo_size.unwrap().get() * 8,
            ),
            None => 0,
        }
    }
}

#[inline]
pub fn pack_variable_data(index: u16, subindex: u8, num_bits: u8) -> u32 {
    ((index as u32) << 16) + ((subindex as u32) << 8) + num_bits as u32
}

#[inline]
pub fn unpack_variable_data(val: u32) -> (u16, u8, u8) {
    ((val >> 16) as u16, (val >> 8) as u8, val as u8)
}

/// Multiple of 100µs
pub struct InhibitTime(pub Option<NonZeroU16>);

impl InhibitTime {
    pub fn us(&self) -> Option<u32> {
        self.0.map(|val| 100 * val.get() as u32)
    }
}

impl From<u16> for InhibitTime {
    fn from(val: u16) -> Self {
        InhibitTime(NonZeroU16::new(val))
    }
}

/*
struct TPDOCanId(Id);

impl TPDOCanId {
    pub fn new(id: Id) -> Option<Self> {
        if let Id::Standard(std_id) = id {
            // CiA 301: 7.3.5 Restricted CAN-IDs
            match std_id.as_raw() {
                0x000..=0x07F
                | 0x101..=0x180
                | 0x581..=0x5FF
                | 0x601..=0x67F
                | 0x6E0..=0x6FF
                | 0x701..=0x77F
                | 0x780..=0x7FF => return None,
                _ => {}
            }
        }
        Some(TPDOCanId(id))
    }
}



/// Multiple of 1ms
struct EventTimer(Option<NonZeroU16>);
*/
