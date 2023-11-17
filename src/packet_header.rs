use std::mem::size_of;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::packet_header::ToPacketHeaderError::InvalidProtocolId;

pub const PROTOCOL_ID: u32 = 694206669;

#[derive(Debug)]
pub struct PacketHeader {
    pub protocol_id: u32,
    pub sequence: u32,
    pub ack: u32,
    pub ack_bitfield: u32
}

#[derive(Debug)]
pub enum ToPacketHeaderError {
    TooSmall {
        expected_min: usize,
        actual: usize
    },
    InvalidProtocolId {
        expected: u32,
        actual: u32
    }
}

impl TryFrom<Bytes> for PacketHeader {
    type Error = ToPacketHeaderError;

    fn try_from(mut bytes: Bytes) -> Result<Self, Self::Error> {
        if bytes.remaining() < 4 * size_of::<u32>() {
            return Err(ToPacketHeaderError::TooSmall {
                expected_min: 4 * size_of::<u32>(),
                actual: bytes.remaining()
            });
        }

        let protocol_id = bytes.get_u32();
        if protocol_id != PROTOCOL_ID {
            return Err(InvalidProtocolId {
                expected: PROTOCOL_ID,
                actual: protocol_id
            });
        }
        
        let sequence = bytes.get_u32();
        let ack = bytes.get_u32();
        let ack_bitfield = bytes.get_u32();

        Ok(PacketHeader { protocol_id, sequence, ack, ack_bitfield })
    }
}

impl From<PacketHeader> for Bytes {
    fn from(value: PacketHeader) -> Self {
        let mut bytes = BytesMut::with_capacity(4 * size_of::<u32>());
        bytes.put_u32(value.protocol_id);
        bytes.put_u32(value.sequence);
        bytes.put_u32(value.ack);
        bytes.put_u32(value.ack_bitfield);
        
        bytes.into()
    }
}
