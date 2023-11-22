use std::collections::BTreeSet;
use bit_field::BitField;

pub struct RemoteSequenceTracker {
    seen_packets: BTreeSet<u32>
}

impl RemoteSequenceTracker {
    const MAX_TRACKED_PACKETS: u32 = u32::BITS;
    
    pub fn new() -> Self {
        Self {
            seen_packets: BTreeSet::new()
        }
    }

    pub fn on_receive(&mut self, remote_sequence: u32) {
        match self.seen_packets.last() {
            // if there's nothing in the set, just insert it
            None => {
                self.seen_packets.insert(remote_sequence);
            }
            // if our highest sequence number seen hasn't hit the max number of tracked packets,
            // OR if it's within the range of numbers we're currently tracking, go ahead and add it
            Some(highest) if *highest < Self::MAX_TRACKED_PACKETS || *highest - Self::MAX_TRACKED_PACKETS < remote_sequence => {
                self.seen_packets.insert(remote_sequence);
            }
            // packet has come in too late and the other side will have already assumed it was dropped.
            // let them handle it as a dropped packet. nothing else for us to do
            Some(_) => {
                // no-op
                return;
            }
        };

        // unwrap won't panic since there's at least one record at this point
        let highest = *self.seen_packets.last().unwrap();
        let minimum = if highest >= Self::MAX_TRACKED_PACKETS {
            highest - Self::MAX_TRACKED_PACKETS
        } else {
            0
        };
        // remove any values below the minimum threshold
        while let Some(lowest) = self.seen_packets.first() {
            // if it's above the minimum, we've removed enough and can complete
            if *lowest >= minimum {
                break;
            }
            // otherwise, remove the lowest sequence number
            let _ = self.seen_packets.pop_first();
        }
    }

    pub fn on_send(&self) -> Option<(u32, u32)> {
        if let Some(highest) = self.seen_packets.last() {
            let mut previous_acks = 0u32;
            let min = if *highest >= Self::MAX_TRACKED_PACKETS {
                highest - Self::MAX_TRACKED_PACKETS
            } else {
                0
            };

            for sequence in self.seen_packets.iter() {
                let index = if *sequence >= min {
                    sequence - min
                } else {
                    panic!("Sequence number {} is less than the minimum {}, which shouldn't happen!", sequence, min);
                };
                // the largest value doesn't go in the bitfield, so ignore it
                if index >= u32::BITS {
                    continue;
                }
                previous_acks.set_bit(index as usize, true);
            }

            Some((*highest, previous_acks))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::RemoteSequenceTracker;

    #[test]
    pub fn empty() {
        let tracker = RemoteSequenceTracker::new();
        assert_eq!(tracker.on_send(), None);
    }

    #[test]
    pub fn a_couple_rounds() {
        let mut tracker = RemoteSequenceTracker::new();
        tracker.on_receive(0);
        assert_eq!(tracker.on_send(), Some((0, 0b1)));
        tracker.on_receive(1);
        assert_eq!(tracker.on_send(), Some((1, 0b11)));
        tracker.on_receive(3);
        assert_eq!(tracker.on_send(), Some((3, 0b1011)));
        tracker.on_receive(2);
        assert_eq!(tracker.on_send(), Some((3, 0b1111)));
    }
    
    #[test]
    pub fn goes_up_to_32() {
        let mut tracker = RemoteSequenceTracker::new();

        for i in 0u32..31 {
            tracker.on_receive(i);
        }
        
        assert_eq!(tracker.on_send(), Some((30, u32::MAX >> 1))); // all bits set except the most significant

        tracker.on_receive(31);

        assert_eq!(tracker.on_send(), Some((31, u32::MAX))); // all bits set

        tracker.on_receive(32);

        assert_eq!(tracker.on_send(), Some((32, u32::MAX))); // all bits still set
    }

    #[test]
    pub fn one_hundred_rounds_every_even() {
        let mut tracker = RemoteSequenceTracker::new();

        for i in 0u32..100 {
            if i % 2 == 0 {
                tracker.on_receive(i);
            }
        }

        assert_eq!(tracker.on_send(), Some((98, 0b01010101010101010101010101010101)));
    }

    #[test]
    pub fn one_hundred_rounds_all() {
        let mut tracker = RemoteSequenceTracker::new();

        for i in 0u32..100 {
            tracker.on_receive(i);
        }

        assert_eq!(tracker.on_send(), Some((99, u32::MAX))); // all bits set
    }
}
