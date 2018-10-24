use canframe::CanFrame;
use futures::sink::Send;
use futures::stream::{SplitSink, SplitStream};
use futures::Future;
use futures::Sink;
use slcan::CanFrameCodec;
use tokio::codec::Framed;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_serial::Serial;

//TODO: make generic
pub struct CanBus {
    writer: SplitSink<Framed<Serial, CanFrameCodec>>,
    reader: SplitStream<Framed<Serial, CanFrameCodec>>,
}

impl CanBus {
    pub fn send(mut self, frame: CanFrame) {
    }
}
