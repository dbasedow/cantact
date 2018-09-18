use std::io;

trait TransportLayer {
    fn receive(data: &[u8]);
    fn send(source_address: u32, target_address: u32, data: &[u8]) -> io::Result<()>;
    fn set_data_layer(layer: Box<impl DataLayer>);
}

trait DataLayer {
    fn send(source_address: u32, target_address: u32, data: &[u8]) -> io::Result<()>;
    fn set_transport_layer(layer: Box<impl TransportLayer>);
}

struct IsoTp {}

enum MessageType {
    Diagnostics,
    RemoteDiagnostics,
}

enum TargetAddressType {
    Physical,
    Functional,
}

enum ResultType {
    //Ok,
    TimeoutA,
    TimeoutBs,
    TimeoutCr,
    WrongSequenceNumber,
    InvalidFlowStatus,
    UnexpectedPDU,
    WaitFrameTimeOverrun,
    BufferOverflow,
    Error,
}

struct Message {}

impl IsoTp {
    fn request_transfer(
        source_address: u32,
        target_address: u32,
        data: &[u8],
    ) -> Result<(), ResultType> {
        Ok(())
    }
}
