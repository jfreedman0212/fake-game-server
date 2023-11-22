use std::collections::{BTreeSet};
use bit_field::BitField;
use crate::net::PacketHeader;

pub struct LocalSequenceTracker {
    local_sequence: u32,
    unacked_packets: BTreeSet<u32>
}

impl LocalSequenceTracker {
    pub fn new() -> Self {
        Self {
            local_sequence: 0,
            unacked_packets: BTreeSet::new()
        }
    }
    
    pub fn on_send(&mut self, ack: u32, ack_bitfield: u32) -> PacketHeader {
        let result = PacketHeader {
            sequence: self.local_sequence,
            ack,
            ack_bitfield
        };
        self.unacked_packets.insert(self.local_sequence);
        // TODO: this ignores when the sequence overflows, and will most likely panic!
        self.local_sequence += 1;
        result
    }
    
    pub fn on_receive(&mut self, packet_header: &PacketHeader) -> Option<BTreeSet<u32>> {
        let mut result = None;

        let min_sequence = if packet_header.ack > u32::BITS {
            packet_header.ack - u32::BITS
        } else {
            packet_header.ack
        };

        if let Some(first) = self.unacked_packets.first() {
            if *first < min_sequence {
                let mut dropped_packets = BTreeSet::new();
                while let Some(first) = self.unacked_packets.first() {
                    if *first < min_sequence {
                        // unwrap is okay since we already know it will be Some
                        dropped_packets.insert(self.unacked_packets.pop_first().unwrap());
                    } else {
                        break;
                    }
                }
                result = Some(dropped_packets);
            }
        }

        self.unacked_packets.retain(|val| {
            let index = Self::get_ack_index_from_sequence(*val, packet_header.ack);
            // if the index is greater or equal than the limit, keep it in the set. no need
            // to remove it yet
            if index >= u32::BIT_LENGTH {
                return true;
            }
            let is_ack = *val == packet_header.ack;
            let is_in_bitfield = packet_header.ack_bitfield.get_bit(index);
            // if it's not the ack field AND it's not in the bitfield, then that packet number
            // has not been received by the other side
            return !is_ack && !is_in_bitfield;
        });
        
        result
    }

    fn get_ack_index_from_sequence(sequence: u32, remote_sequence: u32) -> usize {
        u32::BIT_LENGTH.min(remote_sequence as usize) + sequence as usize - remote_sequence as usize
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeSet;
    use bit_field::BitField;
    use crate::net::PacketHeader;
    use super::LocalSequenceTracker;

    #[test]
    pub fn on_send_increments_sequence() {
        let mut tracker = LocalSequenceTracker::new();
        let packet = tracker.on_send(0, 0);
        assert_eq!(packet.sequence, 0);
        let packet = tracker.on_send(0, 0);
        assert_eq!(packet.sequence, 1);
        let packet = tracker.on_send(0, 0);
        assert_eq!(packet.sequence, 2);
        let packet = tracker.on_send(0, 0);
        assert_eq!(packet.sequence, 3);
    }

    #[test]
    pub fn does_not_drop_packets() {
        let mut tracker = LocalSequenceTracker::new();

        for i in 0u32..10 {
            tracker.on_send(0, 0);
            let mut ack_bitfield = 0u32;
            for j in 0..=i as usize {
                ack_bitfield.set_bit(j, true);
            }
            let in_packet = PacketHeader {
                sequence: i,
                ack: i,
                ack_bitfield
            };
            assert_eq!(tracker.on_receive(&in_packet), None);
        }
    }

    #[test]
    pub fn drops_packets() {
        let mut tracker = LocalSequenceTracker::new();

        for _ in 0..45 {
            tracker.on_send(0, 0);
        }
        
        let in_packet = PacketHeader {
            sequence: 44,
            ack: 44,
            // pretend we haven't gotten ANY packets other than the most recent one
            ack_bitfield: 0
        };
        assert_eq!(tracker.on_receive(&in_packet), Some(BTreeSet::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11])));
    }
}