use canframe::CanFrame;
use futures::prelude::*;
use futures::stream::{SplitSink, SplitStream};
use protocols::transport::general::CanBus;
use slcan::CanFrameCodec;
use std::cmp;
use std::io;
use std::time::{Duration, Instant};
use tokio;
use tokio::codec::{Decoder, Encoder, Framed};
use tokio::prelude::*;
use tokio::timer::Delay;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_serial::Serial;

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

struct IsoTpTransfer {
    //reader: SplitStream<Framed<Serial, CanFrameCodec>>,
    inner: Future<Item = Vec<u8>, Error = ResultType>,
}

struct IsoTpTransferState {
    tx_arbitration_id: u32,
    rx_arbitration_id: u32,
    data: Vec<u8>,
    expect_response: bool,
    done: bool,
}

fn slice_to_frame_payload(data: &[u8]) -> [u8; 8] {
    let max = cmp::min(data.len(), 8);
    let mut res = [0; 8];

    for (i, b) in data[..max].iter().enumerate() {
        res[i] = *b;
    }
    res
}

#[test]
fn test_slice_to_payload() {
    let d = b"abcdefghijklmopqrstuvwxyz";
    let res = slice_to_frame_payload(&d[0..8]);
    assert_eq!(*b"abcdefgh", res);
    let res = slice_to_frame_payload(&d[1..9]);
    assert_eq!(*b"bcdefghi", res);
}
/*
impl IsoTpTransfer {
    fn send(mut self, bus: Framed<Serial, CanFrameCodec>) {
        if self.data.len() <= 7 {
            self.send_single_frame(bus);
        } else {
            self.send_multi_frame();
        }
    }

    fn send_single_frame(mut self, bus: Framed<Serial, CanFrameCodec>) {
        let (writer, reader) = bus.split();
        let t = writer
            .send(CanFrame {
                id: self.tx_arbitration_id,
                rtr: false,
                ext: false,
                length: self.data.len(),
                data: slice_to_frame_payload(&self.data[..]),
            }).and_then(move |writer| {
                if self.expect_response {
                } else {
                    self.done = true;
                    task::current().notify();
                }
                Ok(())
            }).map_err(|e| eprintln!("{}", e));

        tokio::spawn(t);
    }

    fn send_multi_frame(&mut self) {}
}

impl Future for IsoTpTransfer {
    type Item = Vec<u8>;
    type Error = ResultType;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        match self.inner.poll() {
            Ok(Async::Ready(d)) => Ok(Async::Ready(d.clone()))
        }
    }
}

fn send_request(
    bus: Framed<Serial, CanFrameCodec>,
    tx_arbitration_id: u32,
    rx_arbitration_id: u32,
    data: &[u8],
    expect_response: bool,
) -> IsoTpTransfer {
    let mut f = IsoTpTransfer {
        tx_arbitration_id,
        rx_arbitration_id,
        data: data.to_vec(),
        expect_response,
        done: false,
    };
    f.send(bus);
}
*/