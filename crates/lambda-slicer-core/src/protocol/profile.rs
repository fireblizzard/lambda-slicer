//! The demo and network protocol pair this decoder and its exports support.
//! Metadata inspection works on any profile, but decoding or transforming a
//! recording of another profile would apply the wrong grammar

use crate::container::Header;

/// GoldSrc demo protocol of the supported recordings
pub const DEMO_PROTOCOL: i32 = 5;
/// Network protocol of the supported recordings
pub const NETWORK_PROTOCOL: i32 = 48;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("protocol {demo}/{network} is not supported (only {DEMO_PROTOCOL}/{NETWORK_PROTOCOL} is)")]
pub struct UnsupportedProfile {
    pub demo: i32,
    pub network: i32,
}

/// Accept only demo protocol 5 with network protocol 48
pub fn check(header: &Header) -> Result<(), UnsupportedProfile> {
    let profile = (header.demo_protocol, header.network_protocol);
    if profile != (DEMO_PROTOCOL, NETWORK_PROTOCOL) {
        return Err(UnsupportedProfile {
            demo: header.demo_protocol,
            network: header.network_protocol,
        });
    }

    Ok(())
}
