use std::collections::{BTreeSet};
use crate::net::local_sequence_tracker::LocalSequenceTracker;
use crate::net::PacketHeader;
use crate::net::remote_sequence_tracker::RemoteSequenceTracker;

pub struct PacketManager {
    local_sequence: LocalSequenceTracker,
    remote_sequence: RemoteSequenceTracker,
}

impl PacketManager {
    pub fn new() -> Self {
        Self {
            local_sequence: LocalSequenceTracker::new(),
            remote_sequence: RemoteSequenceTracker::new()
        }
    }
    
    pub fn send(&mut self) -> Option<PacketHeader> {
        let (ack, ack_bitfield) = self.remote_sequence.on_send()?;
        let header = self.local_sequence.on_send(ack, ack_bitfield);
        Some(header)
    }
    
    pub fn receive(&mut self, packet_header: &PacketHeader) -> Option<BTreeSet<u32>> {
        self.remote_sequence.on_receive(packet_header.sequence);
        self.local_sequence.on_receive(packet_header)
    }
}

pub enum SendError {
    
}