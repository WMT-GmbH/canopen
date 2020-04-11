use super::*;

#[derive(Debug)]
pub struct SdoServer {
    rx_cobid: u32,
    tx_cobid: u32,
}

impl SdoServer {
    pub fn new(rx_cobid: u32, tx_cobid: u32) -> SdoServer {
        SdoServer { rx_cobid, tx_cobid }
    }

    pub fn on_request(&self, _can_id: u32, data: &[u8]){
        if data.len() != 8 {

        }
        let (command, request) = data.split_first().unwrap();
        let ccs = command & 0xE0;

        let result = match ccs {
            REQUEST_UPLOAD => self.init_upload(request),
            REQUEST_SEGMENT_UPLOAD => self.segmented_upload(command),
            REQUEST_DOWNLOAD => self.init_download(request),
            REQUEST_SEGMENT_DOWNLOAD => self.segmented_download(command, request),
            REQUEST_ABORTED => Ok(()),
            _ => Err(SdoAbortedError{code: 0x05040001}),
        };
        if result.is_err(){
            self.abort(result.unwrap_err())
        }
    }

    fn init_upload(&self, request: &[u8]) -> Result<(), SdoAbortedError>{
        println!("{:?}", request);
        Ok(())
    }

    fn segmented_upload(&self, command: &u8) -> Result<(), SdoAbortedError>{
        println!("{}", command);
        Ok(())
    }

    fn init_download(&self, request: &[u8]) -> Result<(), SdoAbortedError>{
        println!("{:?}", request);
        Ok(())
    }

    fn segmented_download(&self, command: &u8, request: &[u8]) -> Result<(), SdoAbortedError>{
        println!("{}, {:?}", command, request);
        Ok(())
    }

    fn abort(&self, abort_error: SdoAbortedError){
        println!("{}", abort_error);
    }
}