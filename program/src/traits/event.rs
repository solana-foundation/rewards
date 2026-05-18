use alloc::vec::Vec;
use const_crypto::sha2::Sha256;

use crate::events::EVENT_IX_TAG_LE;

/// Length of event discriminator bytes: EVENT_IX_TAG_LE (8) + Anchor discriminator (8)
pub const EVENT_DISCRIMINATOR_LEN: usize = 8 + 8;

/// Compute an Anchor-compatible event discriminator at compile time:
/// `sha256("event:<name>")[..8]`.
///
/// Call this from each event's `EventDiscriminator::DISCRIMINATOR` constant
/// to avoid hand-computing hashes and to stay in sync with the event name.
pub const fn event_discriminator(name: &[u8]) -> [u8; 8] {
    const PREFIX: &[u8] = b"event:";
    let hash = Sha256::new().update(PREFIX).update(name).finalize();
    [hash[0], hash[1], hash[2], hash[3], hash[4], hash[5], hash[6], hash[7]]
}

/// Anchor-compatible event discriminator: `sha256("event:StructName")[..8]`.
///
/// Implementors should derive the constant via [`event_discriminator`]:
/// ```ignore
/// const DISCRIMINATOR: [u8; 8] = event_discriminator(b"ClaimedEvent");
/// ```
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
        const DISCRIMINATOR: [u8; 8] = event_discriminator(b"TestEvent");
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
        // Verify the derived discriminator matches sha256("event:TestEvent")[..8]
        assert_eq!(&bytes[8..16], &TestEvent::DISCRIMINATOR);
    }

    #[test]
    fn test_event_discriminator_matches_anchor_formula() {
        // sha256("event:ClaimedEvent")[..8] — precomputed with
        // `python3 -c "import hashlib; print(list(hashlib.sha256(b'event:ClaimedEvent').digest()[:8]))"`
        const EXPECTED: [u8; 8] = [0x90, 0xac, 0xd1, 0x56, 0x90, 0x57, 0x54, 0x73];
        assert_eq!(event_discriminator(b"ClaimedEvent"), EXPECTED);
    }

    struct TestEventSerialize {
        pub value: u8,
    }

    impl EventDiscriminator for TestEventSerialize {
        const DISCRIMINATOR: [u8; 8] = event_discriminator(b"TestEventSerialize");
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
        assert_eq!(&bytes[8..16], &TestEventSerialize::DISCRIMINATOR);
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
