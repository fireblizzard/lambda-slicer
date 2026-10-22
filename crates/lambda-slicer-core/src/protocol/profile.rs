//! The demo and network protocols this decoder and its exports support.
//! Metadata inspection works on any profile, but decoding or transforming a
//! recording of another profile would apply the wrong grammar
//!
//! The WON release's network protocol 47 and the Steam release's 48 send
//! the same messages in the same form for what their recordings hold, so
//! both decode alike

use crate::container::Header;

/// GoldSrc demo protocol of the supported recordings
pub const DEMO_PROTOCOL: i32 = 5;
/// Network protocols of the supported recordings, the WON release's and the
/// Steam release's
pub const NETWORK_PROTOCOLS: [i32; 2] = [47, 48];

/// Whether recordings of network protocol `network` are supported
pub fn supports_network(network: i32) -> bool {
    NETWORK_PROTOCOLS.contains(&network)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("protocol {demo}/{network} is not supported (only 5/47 and 5/48 are)")]
pub struct UnsupportedProfile {
    pub demo: i32,
    pub network: i32,
}

impl crate::message::Coded for UnsupportedProfile {
    fn code(&self) -> &'static str {
        "protocol-unsupported"
    }

    fn args(&self) -> Vec<(&'static str, crate::message::Arg)> {
        vec![("demo", self.demo.into()), ("network", self.network.into())]
    }
}

/// Accept only demo protocol 5 with network protocol 47 or 48
pub fn check(header: &Header) -> Result<(), UnsupportedProfile> {
    let supported =
        header.demo_protocol == DEMO_PROTOCOL && supports_network(header.network_protocol);
    if !supported {
        return Err(UnsupportedProfile {
            demo: header.demo_protocol,
            network: header.network_protocol,
        });
    }

    Ok(())
}
