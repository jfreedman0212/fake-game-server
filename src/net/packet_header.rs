use std::mem::size_of;
use bytes::{Buf, BufMut};

const PROTOCOL_ID: u32 = 694206669;

const HEADER_SIZE: usize = 4 * size_of::<u32>();

#[derive(Debug)]
pub struct Packet(pub PacketHeader);

#[derive(Debug)]
pub struct PacketHeader {
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

impl Packet {
    pub fn try_deserialize(bytes: &mut impl Buf) -> Result<Self, ToPacketHeaderError> {
        Ok(Packet(PacketHeader::try_deserialize(bytes)?))
    }

    pub fn serialize(&self, buffer: &mut impl BufMut) {
        self.0.serialize(buffer)
    }
}

impl PacketHeader {
    pub fn try_deserialize(bytes: &mut impl Buf) -> Result<Self, ToPacketHeaderError> {
        if bytes.remaining() < HEADER_SIZE {
            return Err(ToPacketHeaderError::TooSmall {
                expected_min: HEADER_SIZE,
                actual: bytes.remaining()
            });
        }

        let protocol_id = bytes.get_u32();
        if protocol_id != PROTOCOL_ID {
            return Err(ToPacketHeaderError::InvalidProtocolId {
                expected: PROTOCOL_ID,
                actual: protocol_id
            });
        }

        let sequence = bytes.get_u32();
        let ack = bytes.get_u32();
        let ack_bitfield = bytes.get_u32();

        Ok(PacketHeader { sequence, ack, ack_bitfield })
    }
    
    pub fn serialize(&self, buffer: &mut impl BufMut) {
        buffer.put_u32(PROTOCOL_ID);
        buffer.put_u32(self.sequence);
        buffer.put_u32(self.ack);
        buffer.put_u32(self.ack_bitfield);
    }
}

#[cfg(test)]
mod test {
    use bytes::{BytesMut};
    use crate::net::{PacketHeader};
    use crate::net::packet_header::HEADER_SIZE;

    #[test]
    pub fn it_works() {
        let packet_header = PacketHeader {
            sequence: 10,
            ack: 100,
            ack_bitfield: 1000
        };
        let mut bytes = BytesMut::with_capacity(HEADER_SIZE);
        packet_header.serialize(&mut bytes);
        
        let parsed_packet_header = PacketHeader::try_deserialize(&mut bytes).unwrap();
        
        assert_eq!(parsed_packet_header.sequence, 10);
        assert_eq!(parsed_packet_header.ack, 100);
        assert_eq!(parsed_packet_header.ack_bitfield, 1000);
    }
}