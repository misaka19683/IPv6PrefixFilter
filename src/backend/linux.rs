use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use std::thread::sleep;
use ipnet::Ipv6Net;
use log::info;
use nftables::batch::Batch;
use nftables::helper::apply_ruleset;
use nftables::schema::{Chain, NfListObject, NfObject, Rule, Table};
use crate::error::FilterError;
use pnet::datalink::NetworkInterface;
use crate::platform::traits::{FilterConfig, FilterMode, PacketProcessor};

pub struct LinuxFilter<'a> {
    pub(crate) queue_num: u32,
    pub(crate) interface_name: Option<NetworkInterface>,
    pub(crate) ruleset: Vec<NfObject<'a>>,
}
impl Debug for LinuxFilter<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LinuxFilter")
            .field("queue_num", &self.queue_num)
            .field("interface_name", &self.interface_name)
            .field("ruleset", &self.ruleset)
            .finish()
    }
}

impl FilterConfig for LinuxFilter<'_> {
    fn init(&mut self) -> Result<(), FilterError> {
        use nftables::{
            batch::Batch,
            helper::apply_ruleset,
        };
        let mut batch=Batch::new();
        batch.add_all(self.ruleset.clone());
        apply_ruleset(&batch.to_nftables()).unwrap();
        Ok(())
    }
    fn cleanup(&self) -> Result<(), FilterError> {
        let mut batch =Batch::new();
        let ruleset=&self.ruleset;
        for obj in ruleset.iter() {
            if let NfObject::ListObject(list_obj)=obj {
                if let NfListObject::Table(_) = list_obj {batch.delete(list_obj.clone())}
            } else { return Err(FilterError::InitError(String::from("failed to delete nftable rulesets!")))}
        }
        apply_ruleset(&batch.to_nftables()).unwrap();

        Ok(())
    }
}
impl LinuxFilter<'_> {
    pub fn new<'a>(queue_num: u32, interface_name: Option<NetworkInterface>) -> Self {
        let table=Table{family:NfFamily::IP6,name: Cow::from("rafilter"),handle:None};
        let chain=Chain{
            family:NfFamily::IP6,
            table:table.name.clone(),
            name: Cow::from("input"),
            _type:Some(NfChainType::Filter),
            hook:Some(NfHook::Input),
            prio:Some(0),
            policy:Some(NfChainPolicy::Accept),
            ..Default::default()
        };
        let mut rule_expr=vec![
            Statement::Match(Match {
                left: Expression::Named(NamedExpression::Payload(Payload::PayloadField(
                    PayloadField {
                        protocol: Cow::from("icmpv6"),
                        field: Cow::from("type"),
                    },
                ))),
                right: Expression::Number(134), // ICMPv6 Router Advertisement
                op: Operator::EQ,
            }),
            Statement::Queue(nftables::stmt::Queue {
                num: Expression::Number(queue_num),
                flags: None,
            }),
        ];
        if let Some (interface_name)=&interface_name {
            rule_expr.insert(0,Statement::Match(Match {
                left:Expression::Named(NamedExpression::Meta(Meta{key:MetaKey::Iifname})),
                right:Expression::String(Cow::from(interface_name.name.clone())),
                op: Operator::EQ,
            }));
        }
        let rule=Rule {family:NfFamily::IP6,table:table.name.clone(),chain:chain.name.clone(), expr:rule_expr.into(),
            comment:Some(Cow::from("Queue ICMPv6 Router Advertisement packets")),
            ..Default::default()
        };
        let ruleset=vec![
            NfObject::ListObject(NfListObject::Table(table)),
            NfObject::ListObject(NfListObject::Chain(chain)),
            NfObject::ListObject(NfListObject::Rule(rule)),
        ];
        Self {
            queue_num,interface_name,ruleset
        }
    }
}

use nfq::{Queue, Verdict};
use nftables::expr::{Expression, Meta, MetaKey, NamedExpression, Payload, PayloadField};
use nftables::stmt::{Match, Operator, Statement};
use nftables::types::{NfChainPolicy, NfChainType, NfFamily, NfHook};
use pnet::packet::icmpv6::Icmpv6Packet;
use pnet::packet::icmpv6::ndp::NdpOptionTypes::PrefixInformation;
use pnet::packet::icmpv6::ndp::RouterAdvertPacket;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::Packet;
use crate::platform::types::ToBytes;

#[derive(Debug, PartialEq)]
pub struct LinuxPacketProcessor {
    pub(crate) filter_mode:FilterMode,
    pub(crate) filter:Vec<Ipv6Net>,
}

impl PacketProcessor for LinuxPacketProcessor {
    fn capture_packet(&mut self) -> Result<(), FilterError> {
        todo!()
    }

    fn analyze_packet(&mut self,data:&[u8]) -> Result<bool, FilterError> {
        let ipv6_packet=if let Some(ipv6_packet)=Ipv6Packet::new(data) {ipv6_packet} else { return Ok(true) };
        let icmpv6_packet=if let Some(icmpv6_packet)=Icmpv6Packet::new(ipv6_packet.payload()){icmpv6_packet} else { return Ok(true) };
        let ra_packet=if let Some(ra_packet)=RouterAdvertPacket::new(icmpv6_packet.packet()) {ra_packet} else {return Ok(true)};

        for op in ra_packet.get_options() {
            if op.option_type !=PrefixInformation {continue;}

            let option_raw=op.to_bytes();
            let pfi=if let Some(pfi)=crate::platform::types::PrefixInformationPacket::new(&option_raw){pfi}else { continue; };

            if pfi.payload().len()!=16 { continue; }
            else {
                let array:[u8;16]=pfi.payload().try_into().unwrap();
                let ipv6addr=std::net::Ipv6Addr::from(array);
                info!("Recived an IPv6 Prefix: {}", ipv6addr);
            };

            let is_prefix_in_list=self.filter.iter().any(|prefix| {prefix.addr().octets()==pfi.payload()});
            let verdict=match (self.filter_mode,is_prefix_in_list) {
                (false,false)=>{ Ok(true)},//黑名单模式，接受不在名单上的包
                (true,true)=>{ Ok(true)},//白名单模式，接受在名单上的包
                _ =>{Ok(false)},
            };
            return verdict;
        }
        Ok(true) //todo!(写好windows平台的代码后可以将这个函数搬到公共trait上去);
    }

    fn run(&mut self) -> Result<(), FilterError> {
        let mut queue = Queue::open().map_err(|e| FilterError::InitError(e.to_string())).unwrap();
        queue.bind(0).unwrap();
        queue.set_nonblocking(true);
        loop {
            let message =queue.recv();
            if let Ok(mut message)=message {
                if let Ok(result)= self.analyze_packet(message.get_payload()) {
                    message.set_verdict(if result { Verdict::Accept } else { Verdict::Drop });
                    queue.verdict(message).unwrap();
                }
            } else { sleep(std::time::Duration::from_millis(50)); }

        }

        Ok(())
    }
}

