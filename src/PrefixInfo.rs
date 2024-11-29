use pnet_macros::packet;
use pnet_macros_support::types::*;
use pnet::packet::icmpv6::ndp::NdpOptionType;


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