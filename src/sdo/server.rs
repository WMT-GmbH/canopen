use alloc::vec::Vec;

use super::*;
use crate::objectdictionary::{DataStore, Object, ObjectDictionary, Variable};
use crate::sdo::errors::SDOAbortCode;
use crate::Network;

type RequestResult = Result<Option<[u8; 8]>, SDOAbortCode>;

#[derive(Default)]
struct State {
    index: u16,
    subindex: u8,
    toggle_bit: u8,
    buffer: Vec<u8>,
}

pub struct SdoServer<'a, N: Network> {
    _rx_cobid: u32,
    tx_cobid: u32,
    network: &'a N,
    od: &'a ObjectDictionary,
    state: State,
}

impl<'a, N: Network> SdoServer<'a, N> {
    pub fn new(rx_cobid: u32, tx_cobid: u32, network: &'a N, od: &'a ObjectDictionary) -> Self {
        SdoServer {
            _rx_cobid: rx_cobid,
            tx_cobid,
            network,
            od,
            state: State::default(),
        }
    }

    pub fn on_request(&mut self, data: &[u8; 8], data_store: &mut DataStore) {
        let ccs = data[0] & 0xE0;

        let result = match ccs {
            REQUEST_UPLOAD => self.init_upload(data, data_store),
            REQUEST_SEGMENT_UPLOAD => self.segmented_upload(data[0]),
            REQUEST_DOWNLOAD => self.init_download(data, data_store),
            REQUEST_SEGMENT_DOWNLOAD => self.segmented_download(data, data_store),
            REQUEST_ABORTED => Ok(None),
            _ => Err(SDOAbortCode::CommandSpecifierError),
        };
        match result {
            Ok(None) => {}
            Ok(Some(response)) => self.send_response(response),
            Err(abort_code) => self.abort(abort_code),
        };
    }

    fn init_upload(&mut self, request: &[u8; 8], data_store: &DataStore) -> RequestResult {
        self.state.index = ((request[2] as u16) << 8) + request[1] as u16;
        self.state.subindex = request[3];

        let data = self.get_data(data_store)?;
        let mut res_command = RESPONSE_UPLOAD | SIZE_SPECIFIED;
        let mut response = [0; 8];

        let size = data.len();
        if size <= 4 {
            res_command |= EXPEDITED;
            res_command |= (4 - size as u8) << 2;
            response[4..4 + size].copy_from_slice(&data);
        } else {
            response[4..].copy_from_slice(&(size as u32).to_le_bytes());
            self.state.buffer = data;
            self.state.toggle_bit = 0;
        }

        response[0] = res_command;
        response[1..3].copy_from_slice(&self.state.index.to_le_bytes());
        response[3] = self.state.subindex;

        Ok(Some(response))
    }

    fn segmented_upload(&mut self, command: u8) -> RequestResult {
        if command & TOGGLE_BIT != self.state.toggle_bit {
            return Err(SDOAbortCode::ToggleBitNotAlternated);
        }

        let size = self.state.buffer.len().min(7);
        let data: Vec<u8> = self.state.buffer.drain(..size).collect();
        let mut res_command = RESPONSE_SEGMENT_UPLOAD;
        res_command |= self.state.toggle_bit; // add toggle bit
        res_command |= (7 - size as u8) << 1; // add nof bytes not used

        if self.state.buffer.is_empty() {
            res_command |= NO_MORE_DATA; // nothing left in buffer
        }

        self.state.toggle_bit ^= TOGGLE_BIT;

        let mut response = [0; 8];
        response[0] = res_command;
        response[1..1 + size].copy_from_slice(&data);
        Ok(Some(response))
    }

    fn init_download(&mut self, request: &[u8; 8], data_store: &mut DataStore) -> RequestResult {
        // TODO check if writable
        self.state.index = ((request[2] as u16) << 8) + request[1] as u16;
        self.state.subindex = request[3];

        let command = request[0];
        if command & EXPEDITED != 0 {
            let size = match command & SIZE_SPECIFIED {
                0 => 4,
                _ => 4 - ((command >> 2) & 0x3) as usize,
            };
            self.set_data(request[4..4 + size].to_vec(), data_store)?;
        } else {
            self.state.buffer.clear();
            self.state.toggle_bit = 0;
        }

        // ---------- TODO optimize
        let mut response = [0; 8];
        response[0] = RESPONSE_DOWNLOAD;
        response[1] = self.state.index as u8;
        response[2] = (self.state.index >> 8) as u8;
        response[3] = self.state.subindex;
        // ----------
        Ok(Some(response))
    }

    fn segmented_download(
        &mut self,
        request: &[u8; 8],
        data_store: &mut DataStore,
    ) -> RequestResult {
        let command = request[0];
        if command & TOGGLE_BIT != self.state.toggle_bit {
            return Err(SDOAbortCode::ToggleBitNotAlternated);
        }
        let last_byte = (8 - ((command >> 1) & 0x7)) as usize;
        self.state.buffer.extend_from_slice(&request[1..last_byte]);

        if command & NO_MORE_DATA != 0 {
            self.set_data(self.state.buffer.to_vec(), data_store)?;
        }

        let res_command = RESPONSE_SEGMENT_DOWNLOAD | self.state.toggle_bit;
        let response = [res_command, 0, 0, 0, 0, 0, 0, 0];
        self.state.toggle_bit ^= TOGGLE_BIT;

        Ok(Some(response))
    }

    fn abort(&mut self, abort_error: SDOAbortCode) {
        let [index_lo, index_hi] = self.state.index.to_le_bytes();
        let subindex = self.state.subindex;
        let code: [u8; 4] = (abort_error as u32).to_le_bytes();
        let data: [u8; 8] = [
            RESPONSE_ABORTED,
            index_lo,
            index_hi,
            subindex,
            code[0],
            code[1],
            code[2],
            code[3],
        ];

        self.send_response(data);
    }

    fn send_response(&mut self, data: [u8; 8]) {
        self.network.send_message(self.tx_cobid, data);
    }

    pub fn get_data(&self, data_store: &DataStore) -> Result<Vec<u8>, SDOAbortCode> {
        let variable = self.find_variable()?;
        data_store.get_data(variable)
    }

    pub fn set_data(
        &mut self,
        data: Vec<u8>,
        data_store: &mut DataStore,
    ) -> Result<(), SDOAbortCode> {
        // TODO check if writable
        let variable = self.find_variable()?;
        data_store.set_data(variable, data)
    }

    fn find_variable(&self) -> Result<&'a Variable, SDOAbortCode> {
        let object = self
            .od
            .get(self.state.index)
            .ok_or(SDOAbortCode::ObjectDoesNotExist)?;

        match object {
            Object::Variable(variable) => Ok(variable),
            Object::Array(array) => Ok(array
                .get(self.state.subindex)
                .ok_or(SDOAbortCode::SubindexDoesNotExist)?),
            Object::Record(record) => Ok(record
                .get(self.state.subindex)
                .ok_or(SDOAbortCode::SubindexDoesNotExist)?),
        }
    }
}

impl State {}
