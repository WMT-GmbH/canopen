use core::cell::Cell;
use core::num::{NonZeroU16, NonZeroUsize};

use crate::node::NodeId;
use embedded_can::{ExtendedId, Id, StandardId};

use crate::objectdictionary::datalink::{
    DataLink, ReadStream, ReadStreamData, UsedReadStream, WriteStream,
};
use crate::objectdictionary::{ObjectDictionaryExt, Variable};
use crate::sdo::SDOAbortCode;
use crate::ObjectDictionary;

pub struct TPDO<'a> {
    pub od: ObjectDictionary<'a>,
    pub com: PDOCommunicationParameter,
    pub map: MappedVariables<'a>,
    pub num_mapped_variables: Cell<u8>,
    pub cob_id_update_func: fn(CobId, CobId) -> Result<CobId, InvalidCobId>,
}

impl<'a> TPDO<'a> {
    #[inline]
    pub fn new_default(
        tpdo: DefaultTPDO,
        node_id: NodeId,
        od: ObjectDictionary<'a>,
        cob_id_update_func: fn(CobId, CobId) -> Result<CobId, InvalidCobId>,
    ) -> Self {
        let com = PDOCommunicationParameter::new(tpdo.cob_id(node_id, false, false));
        TPDO::new(od, com, MappedVariables::default(), cob_id_update_func)
    }

    #[inline]
    pub fn new(
        od: ObjectDictionary<'a>,
        com: PDOCommunicationParameter,
        map: MappedVariables<'a>,
        cob_id_update_func: fn(CobId, CobId) -> Result<CobId, InvalidCobId>,
    ) -> Self {
        TPDO {
            od,
            com,
            map,
            num_mapped_variables: Cell::new(0),
            cob_id_update_func,
        }
    }

    pub fn create_frame<F: embedded_can::Frame>(&self) -> Result<F, SDOAbortCode> {
        let mut buf = [0; 8];
        let mut frame_len = 0;
        let mut read_stream_data = ReadStreamData {
            index: 0,
            subindex: 0,
            buf: &mut buf,
            total_bytes_read: &mut frame_len,
            is_last_segment: false,
        };
        let mut read_stream_ref = &mut read_stream_data;
        for i in 0..self.num_mapped_variables.get() as usize {
            if let Some(variable) = self.map.0[i].get() {
                read_stream_ref.index = variable.index;
                read_stream_ref.subindex = variable.subindex;
                read_stream_ref = variable.read(ReadStream(read_stream_ref))?.0;
            }
        }

        Ok(F::new(self.com.cob_id().id, &buf[0..frame_len]).unwrap())
    }

    pub fn map_variable(&self, slot: u8, variable: &'a Variable<'a>) -> Result<(), ()> {
        // TODO check sizes
        if variable.size().is_some() {
            self.map.0[slot as usize].set(Some(variable));
            Ok(())
        } else {
            Err(())
        }
    }
}

impl DataLink for TPDO<'_> {
    fn size(&self, index: u16, subindex: u8) -> Option<NonZeroUsize> {
        match index {
            0x1800..=0x19FF => match subindex {
                1 => NonZeroUsize::new(4),
                3 | 5 => NonZeroUsize::new(2),
                _ => NonZeroUsize::new(1),
            },
            0x1A00..=0x1BFF => NonZeroUsize::new(4),
            _ => unreachable!(),
        }
    }

    fn read<'rs>(&self, read_stream: ReadStream<'rs>) -> Result<UsedReadStream<'rs>, SDOAbortCode> {
        match read_stream.index {
            0x1800..=0x19FF => match read_stream.subindex {
                1 => self.com.cob_id.read(read_stream),
                2 => self.com.transmission_type.read(read_stream),
                3 => self.com.inhibit_time.read(read_stream),
                5 => self.com.event_timer.read(read_stream),
                6 => self.com.sync_start_value.read(read_stream),
                _ => unreachable!(),
            },
            0x1A00..=0x1BFF => match read_stream.subindex {
                0 => self.num_mapped_variables.read(read_stream),
                n => self.map.get_map_data_packed(n).read(read_stream),
            },
            _ => unreachable!(),
        }
    }

    fn write(&self, write_stream: &WriteStream<'_>) -> Result<(), SDOAbortCode> {
        // if currently valid, the only allowed write is to the valid bit
        if self.com.cob_id().valid && (write_stream.index > 0x2000 || write_stream.subindex != 1) {
            return Err(SDOAbortCode::DeviceStateError);
        }

        match write_stream.index {
            0x1800..=0x19FF => match write_stream.subindex {
                1 => {
                    let new_cob_id = Cell::new(0);
                    new_cob_id.write(write_stream)?;
                    let new_cob_id = (self.cob_id_update_func)(
                        self.com.cob_id(),
                        CobId::from(new_cob_id.get()),
                    )?;

                    self.com.cob_id.set(new_cob_id.into());
                    Ok(())
                }
                2 => self.com.transmission_type.write(write_stream),
                3 => self.com.inhibit_time.write(write_stream),
                5 => self.com.event_timer.write(write_stream),
                6 => self.com.sync_start_value.write(write_stream),
                _ => unreachable!(),
            },
            0x1A00..=0x1BFF => {
                if write_stream.subindex == 0 {
                    return self.num_mapped_variables.write(write_stream);
                }
                if self.num_mapped_variables.get() > 0 {
                    // num_mapped_objects needs to be set to 0 before updating mapping
                    return Err(SDOAbortCode::DeviceStateError);
                }
                if let Ok(data) = write_stream.new_data.try_into() {
                    let data = <u32>::from_le_bytes(data);
                    let (index, subindex, num_bits) = unpack_variable_data(data);

                    match self.od.find(index, subindex) {
                        Ok(variable) => {
                            if let Some(size) = variable.size() {
                                if size.get() * 8 != num_bits {
                                    return Err(SDOAbortCode::ObjectCannotBeMapped);
                                }
                            } else {
                                return Err(SDOAbortCode::ObjectCannotBeMapped);
                            }

                            self.map_variable(write_stream.subindex, variable).unwrap(); // TODO unwrap
                            self.map.0[write_stream.subindex as usize - 1].set(Some(variable));
                        }
                        Err(_) => return Err(SDOAbortCode::ObjectDoesNotExist),
                    }
                }

                Ok(())
            }
            _ => unreachable!(),
        }
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

impl From<InvalidCobId> for SDOAbortCode {
    fn from(_: InvalidCobId) -> Self {
        SDOAbortCode::InvalidValue
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
    cob_id: Cell<u32>,
    transmission_type: Cell<u8>,
    inhibit_time: Cell<u16>,
    event_timer: Cell<u16>,
    sync_start_value: Cell<u8>,
}

impl PDOCommunicationParameter {
    pub fn new(cob_id: CobId) -> Self {
        PDOCommunicationParameter {
            cob_id: Cell::new(cob_id.into()),
            transmission_type: Cell::new(0),
            inhibit_time: Cell::new(0),
            event_timer: Cell::new(0),
            sync_start_value: Cell::new(0),
        }
    }

    pub fn cob_id(&self) -> CobId {
        self.cob_id.get().into()
    }

    pub fn inhibit_time(&self) -> InhibitTime {
        self.inhibit_time.get().into()
    }
}

#[derive(Default)]
pub struct MappedVariables<'a>([Cell<Option<&'a Variable<'a>>>; 8]);

impl MappedVariables<'_> {
    #[inline]
    fn get_map_data_packed(&self, num: u8) -> u32 {
        match self.0[num as usize - 1].get() {
            Some(variable) => pack_variable_data(
                variable.index,
                variable.subindex,
                variable.size().unwrap().get() * 8,
            ),
            None => 0,
        }
    }
}

#[inline]
fn pack_variable_data(index: u16, subindex: u8, num_bits: usize) -> u32 {
    ((index as u32) << 16) + ((subindex as u32) << 8) + num_bits as u32
}

#[inline]
fn unpack_variable_data(val: u32) -> (u16, u8, usize) {
    ((val >> 16) as u16, (val >> 8) as u8, val as usize & 0xFF)
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
