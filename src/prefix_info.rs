use pnet::packet::icmpv6::ndp::{NdpOption, NdpOptionType};
use pnet_macros::packet;
use pnet_macros_support::types::*;

/// Prefix Information Option [RFC 4861 § 4.6.2]
///
/// ```text
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |     Type      |    Length     | Prefix Length |L|A| Reserved1 |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                         Valid Lifetime                        |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                       Preferred Lifetime                      |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                           Reserved2                           |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                                                               |
/// +                                                               +
/// |                                                               |
/// +                            Prefix                             +
/// |                                                               |
/// +                                                               +
/// |                                                               |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
///
/// [RFC 4861 § 4.6.2]: https://tools.ietf.org/html/rfc4861#section-4.6.2
#[allow(dead_code)]
#[packet]
pub struct PrefixInformation {
    #[construct_with(u8)]
    pub option_type: NdpOptionType,
    #[construct_with(u8)]
    pub length: u8,
    pub prefix_length: u8,
    pub flag: u8,
    pub valid_lifetime: u32be,
    pub preferred_lifetime: u32be,
    pub reserved: u32be,
    #[payload]
    pub prefix: Vec<u8>,
}

pub trait ToBytes {
    fn to_bytes(&self) -> Vec<u8>;
}

impl ToBytes for NdpOption {
    fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity((self.length * 8).into());
        result.push(self.option_type.0);
        result.push(self.length);
        result.extend(&self.data);
        return result;
    }
}
