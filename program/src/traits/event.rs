use alloc::vec::Vec;

use crate::events::EVENT_IX_TAG_LE;

/// Length of event discriminator bytes: EVENT_IX_TAG_LE (8) + Anchor discriminator (8)
pub const EVENT_DISCRIMINATOR_LEN: usize = 8 + 8;

/// Anchor-compatible event discriminator: `sha256("event:StructName")[..8]`
pub trait EventDiscriminator {
    const DISCRIMINATOR: [u8; 8];

    #[inline(always)]
    fn discriminator_bytes() -> Vec<u8> {
        let mut data = Vec::with_capacity(EVENT_DISCRIMINATOR_LEN);
        data.extend_from_slice(EVENT_IX_TAG_LE);
        data.extend_from_slice(&Self::DISCRIMINATOR);
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
        const DISCRIMINATOR: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
    }

    #[test]
    fn test_discriminator_bytes_length() {
        let bytes = TestEvent::discriminator_bytes();
        assert_eq!(bytes.len(), EVENT_DISCRIMINATOR_LEN);
        assert_eq!(bytes.len(), 16);
    }

    #[test]
    fn test_discriminator_bytes_prefix() {
        let bytes = TestEvent::discriminator_bytes();
        assert_eq!(&bytes[..8], EVENT_IX_TAG_LE);
        assert_eq!(&bytes[8..16], &[1, 2, 3, 4, 5, 6, 7, 8]);
    }

    struct TestEventSerialize {
        pub value: u8,
    }

    impl EventDiscriminator for TestEventSerialize {
        const DISCRIMINATOR: [u8; 8] = [10, 20, 30, 40, 50, 60, 70, 80];
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
        assert_eq!(&bytes[8..16], &[10, 20, 30, 40, 50, 60, 70, 80]);
        assert_eq!(bytes[16], 123);
    }

    #[test]
    fn test_event_serialize_to_bytes_inner() {
        let event = TestEventSerialize { value: 200 };
        let bytes = event.to_bytes_inner();
        assert_eq!(bytes.len(), 1);
        assert_eq!(bytes[0], 200);
    }
}
