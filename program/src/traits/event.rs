use alloc::vec::Vec;

use crate::events::EVENT_IX_TAG_LE;

/// Length of event discriminator bytes (EVENT_IX_TAG_LE + discriminator byte)
pub const EVENT_DISCRIMINATOR_LEN: usize = 8 + 1;

/// Event discriminator values for this program
#[repr(u8)]
pub enum EventDiscriminators {
    Claimed = 0,
    DistributionClosed = 1,
    DistributionCreated = 2,
    RecipientAdded = 3,
    ClaimClosed = 4,
    RecipientRevoked = 5,
    RewardDistributed = 6,
    OptedIn = 7,
    OptedOut = 8,
    BalanceSynced = 9,
}

/// Event discriminator with Anchor-compatible prefix
pub trait EventDiscriminator {
    /// Event discriminator byte
    const DISCRIMINATOR: u8;

    /// Full discriminator bytes including EVENT_IX_TAG_LE prefix
    #[inline(always)]
    fn discriminator_bytes() -> Vec<u8> {
        let mut data = Vec::with_capacity(EVENT_DISCRIMINATOR_LEN);
        data.extend_from_slice(EVENT_IX_TAG_LE);
        data.push(Self::DISCRIMINATOR);
        data
    }
}

/// Event serialization
pub trait EventSerialize: EventDiscriminator {
    /// Serialize event data (without discriminator)
    fn to_bytes_inner(&self) -> Vec<u8>;

    /// Serialize with full discriminator prefix
    #[inline(always)]
    fn to_bytes(&self) -> Vec<u8> {
        let mut data = Self::discriminator_bytes();
        data.extend_from_slice(&self.to_bytes_inner());
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestEvent;

    impl EventDiscriminator for TestEvent {
        const DISCRIMINATOR: u8 = 42;
    }

    #[test]
    fn test_discriminator_bytes_length() {
        let bytes = TestEvent::discriminator_bytes();
        assert_eq!(bytes.len(), EVENT_DISCRIMINATOR_LEN);
    }

    #[test]
    fn test_discriminator_bytes_prefix() {
        let bytes = TestEvent::discriminator_bytes();
        assert_eq!(&bytes[..8], EVENT_IX_TAG_LE);
        assert_eq!(bytes[8], 42);
    }

    struct TestEventSerialize {
        pub value: u8,
    }

    impl EventDiscriminator for TestEventSerialize {
        const DISCRIMINATOR: u8 = 99;
    }

    impl EventSerialize for TestEventSerialize {
        fn to_bytes_inner(&self) -> Vec<u8> {
            alloc::vec![self.value]
        }
    }

    #[test]
    fn test_event_serialize_to_bytes() {
        let event = TestEventSerialize { value: 123 };
        let bytes = event.to_bytes();
        assert_eq!(bytes.len(), EVENT_DISCRIMINATOR_LEN + 1);
        assert_eq!(&bytes[..8], EVENT_IX_TAG_LE);
        assert_eq!(bytes[8], 99);
        assert_eq!(bytes[9], 123);
    }

    #[test]
    fn test_event_serialize_to_bytes_inner() {
        let event = TestEventSerialize { value: 200 };
        let bytes = event.to_bytes_inner();
        assert_eq!(bytes.len(), 1);
        assert_eq!(bytes[0], 200);
    }
}
