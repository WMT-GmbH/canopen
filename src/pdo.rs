use core::cell::Cell;
use core::num::NonZeroUsize;

use embedded_can::{ExtendedId, Id, StandardId};

use crate::objectdictionary::datalink::{DataLink, ReadStream, WriteStream};
use crate::objectdictionary::{ObjectDictionaryExt, Variable};
use crate::sdo::SDOAbortCode;
use crate::ObjectDictionary;

pub struct TPDO<'a> {
    pub od: ObjectDictionary<'a>,
    pub com: PDOCommunicationParameter,
    pub map: MappedObjects<'a>,
    pub num_mapped_objects: Cell<u8>,
    pub buf: [u8; 8],
    pub cob_id_update_func: fn(CobId, CobId) -> Result<CobId, ()>,
}

impl<'a> TPDO<'a> {
    #[inline]
    pub fn new_default(
        tpdo: DefaultTPDO,
        node_id: u8,
        od: ObjectDictionary<'a>,
        cob_id_update_func: fn(CobId, CobId) -> Result<CobId, ()>,
    ) -> Self {
        let com = PDOCommunicationParameter::new(tpdo.cob_id(node_id, false, false));
        TPDO::new(od, com, MappedObjects::default(), cob_id_update_func)
    }

    #[inline]
    pub fn new(
        od: ObjectDictionary<'a>,
        com: PDOCommunicationParameter,
        map: MappedObjects<'a>,
        cob_id_update_func: fn(CobId, CobId) -> Result<CobId, ()>,
    ) -> Self {
        TPDO {
            od,
            com,
            map,
            num_mapped_objects: Cell::new(0),
            buf: [0; 8],
            cob_id_update_func,
        }
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
                _ => NonZeroUsize::new(1),
            },
            0x1A00..=0x1BFF => NonZeroUsize::new(4),
            _ => unreachable!(),
        }
    }

    fn read(&self, read_stream: &mut ReadStream<'_>) -> Result<(), SDOAbortCode> {
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
                0 => self.num_mapped_objects.read(read_stream),
                n => self.map.get_map_data_packed(n).read(read_stream),
            },
            _ => unreachable!(),
        }
    }

    fn write(&self, write_stream: &WriteStream<'_>) -> Result<(), SDOAbortCode> {
        // if currently valid, the only allowed write is to the valid bit
        if self.com.cob_id().valid && (write_stream.index > 0x2000 || write_stream.subindex != 1) {
            return Err(SDOAbortCode::UnsupportedAccess);
        }

        match write_stream.index {
            0x1800..=0x19FF => match write_stream.subindex {
                1 => {
                    let new_cob_id = Cell::new(0);
                    new_cob_id.write(write_stream)?;
                    if let Ok(cob_id) =
                        (self.cob_id_update_func)(self.com.cob_id(), CobId::from(new_cob_id.get()))
                    {
                        self.com.cob_id.set(cob_id.into());
                        Ok(())
                    } else {
                        Err(SDOAbortCode::InvalidValue)
                    }
                }
                2 => self.com.transmission_type.write(write_stream),
                3 => self.com.inhibit_time.write(write_stream),
                5 => self.com.event_timer.write(write_stream),
                6 => self.com.sync_start_value.write(write_stream),
                _ => unreachable!(),
            },
            0x1A00..=0x1BFF => {
                if write_stream.subindex == 0 {
                    return self.num_mapped_objects.write(write_stream);
                }
                if self.num_mapped_objects.get() > 0 {
                    // num_mapped_objects needs to be set to 0 before updating mapping
                    return Err(SDOAbortCode::UnsupportedAccess);
                }
                if let Ok(data) = write_stream.new_data.try_into() {
                    let data = <u32>::from_le_bytes(data);
                    let (index, subindex, size) = unpack_variable_data(data);

                    match self.od.find(index, subindex) {
                        Ok(variable) => {
                            if variable.size() != NonZeroUsize::new(size) {
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
    pub fn cob_id(self, node_id: u8, valid: bool, rtr: bool) -> CobId {
        CobId {
            valid,
            rtr,
            id: Id::Standard(unsafe {
                StandardId::new_unchecked(0x180 + 0x100 * self as u16 + node_id as u16)
            }),
        }
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
    inhibit_time: Cell<u8>,
    event_timer: Cell<u8>,
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
}

#[derive(Default)]
pub struct MappedObjects<'a>([Cell<Option<&'a Variable<'a>>>; 8]);

impl MappedObjects<'_> {
    #[inline]
    fn get_map_data_packed(&self, num: u8) -> u32 {
        match self.0[num as usize - 1].get() {
            Some(variable) => pack_variable_data(
                variable.index,
                variable.subindex,
                variable.size().unwrap().get(),
            ),
            None => 0,
        }
    }
}

#[inline]
fn pack_variable_data(index: u16, subindex: u8, size: usize) -> u32 {
    ((index as u32) << 16) + ((subindex as u32) << 8) + size as u32
}

#[inline]
fn unpack_variable_data(val: u32) -> (u16, u8, usize) {
    ((val >> 16) as u16, (val >> 8) as u8, val as usize & 0xFF)
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


/// Multiple of 100µs
struct InhibitTime(Option<NonZeroU8>);

/// Multiple of 1ms
struct EventTimer(Option<NonZeroU8>);
*/
