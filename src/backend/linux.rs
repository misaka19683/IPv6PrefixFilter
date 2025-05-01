use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use ipnet::Ipv6Net;
use log::info;
use nftables::batch::Batch;
use nftables::helper::apply_ruleset;
use nftables::schema::{NfListObject, NfObject};
use crate::error::FilterError;
use pnet::datalink::NetworkInterface;
use crate::platform::traits::{FilterConfig, FilterMode, PacketProcessor};

pub struct LinuxFilter<'a> {
    pub(crate) queue_num: u32,
    pub(crate) interface_name: Option<NetworkInterface>,
    pub(crate) ruleset: Option<Vec<NfObject<'a>>>,
}
impl Debug for LinuxFilter<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl FilterConfig for LinuxFilter<'_> {
    fn init(&mut self) -> Result<(), FilterError> {
        use nftables::{
            batch::Batch,
            expr::{Expression, NamedExpression, Meta, MetaKey, Payload, PayloadField},
            helper::apply_ruleset,
            schema::{Chain, NfListObject, NfObject, Rule, Table},
            stmt::{Match, Operator, Queue, Statement},
            types::{NfChainPolicy, NfChainType, NfFamily, NfHook},
        };
        fn create_nftables_objects<'a>(queue_num: u32, interface_name: Option<NetworkInterface>) -> Vec<NfObject<'a>> {
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
                Statement::Queue(Queue {
                    num: Expression::Number(queue_num),
                    flags: None,
                }),
            ];
            if let Some (interface_name)=interface_name {
                rule_expr.insert(0,Statement::Match(Match {
                    left:Expression::Named(NamedExpression::Meta(Meta{key:MetaKey::Iifname})),
                    right:Expression::String(Cow::from(interface_name.name)),
                    op: Operator::EQ,
                }));
            }
            let rule=Rule {family:NfFamily::IP6,table:table.name.clone(),chain:chain.name.clone(), expr:rule_expr.into(),
                comment:Some(Cow::from("Queue ICMPv6 Router Advertisement packets")),
                ..Default::default()
            };
            vec![
                NfObject::ListObject(*Box::new(NfListObject::Table(table))),
                NfObject::ListObject(*Box::new(NfListObject::Chain(chain))),
                NfObject::ListObject(*Box::new(NfListObject::Rule(rule))),
            ]
        }
        let ruleset=create_nftables_objects(self.queue_num,self.interface_name.clone());
        let mut batch=Batch::new();
        batch.add_all(ruleset.clone());
        self.ruleset=Some(ruleset);
        apply_ruleset(&batch.to_nftables()).unwrap();
        Ok(())
    }
    fn cleanup(&self) -> Result<(), FilterError> {
        let mut batch =Batch::new();
        let ruleset=if let Some(ruleset)=self.ruleset.clone() {ruleset} else { return Err(FilterError::InitError(String::from("failed to remove nft rules"))) };
        for obj in ruleset.iter() {
            if let NfObject::ListObject(list_obj)=obj {
                if let NfListObject::Table(_) = list_obj {batch.delete(list_obj.clone())}
            } else { return Err(FilterError::InitError(String::from("failed to delete nftable rulesets!")))}
        }
        apply_ruleset(&batch.to_nftables()).unwrap();
        todo!("此处无法独立清除nft规则，需要修改");
        Ok(())
    }
}


use nfq::{Queue, Verdict};
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
            let mut message =queue.recv().unwrap();
            if let Ok(result)= self.analyze_packet(message.get_payload()) {
                message.set_verdict(if result { Verdict::Accept } else { Verdict::Drop });
                queue.verdict(message).unwrap();
            }
        }

        // Ok(())
    }
}

