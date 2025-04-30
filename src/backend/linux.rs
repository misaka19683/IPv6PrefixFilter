use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use std::net::IpAddr;
use ipnet::Ipv6Net;
use nftables::batch::Batch;
use nftables::helper::apply_ruleset;
use nftables::schema::{NfListObject, NfObject};
use crate::error::FilterError;
use pnet::datalink::NetworkInterface;
use crate::platform::traits::{FilterAction, FilterConfig, PacketProcessor};

pub struct LinuxFilter<'a> {
    queue_num: u32,
    interface_name: Option<NetworkInterface>,
    ruleset: Vec<NfObject<'a>>,
}
impl<'a> Debug for LinuxFilter<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<'a> crate::platform::traits::FilterConfig for LinuxFilter<'a> {
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
        self.ruleset=ruleset;
        apply_ruleset(&batch.to_nftables()).unwrap();
        Ok(())
    }
    fn cleanup(&self) -> Result<(), FilterError> {
        let mut batch =Batch::new();
        for obj in self.ruleset.iter() {
            if let NfObject::ListObject(list_obj)=obj {
                if let NfListObject::Table(_) = list_obj {batch.delete(list_obj.clone())}
            } else { return Err(FilterError::InitError(String::from("failed to delete nftable rulesets!")))}
        }
        apply_ruleset(&batch.to_nftables()).unwrap();
        Ok(())
    }
}


use nfq::Queue;
#[derive(Debug, PartialEq)]
pub struct LinuxPacketProcessor {
    filter_mode:bool,
    packet:Option<IpAddr>,
    filter:Vec<Ipv6Net>,
    filter_action:FilterAction
}
impl PacketProcessor for LinuxPacketProcessor {
    fn capture_packet(&mut self) -> Result<(), FilterError> {
        todo!()
    }

    fn analyze_packet(&mut self) -> Result<(), FilterError> {
        todo!()
    }
}

struct LinuxRuntime<A,B> {
    init_module:A,
    packet_process:B,
}
impl<'a> LinuxRuntime<LinuxFilter<'a>, LinuxPacketProcessor> {
    fn process() {
        let mut filter =LinuxFilter{
            queue_num:0,interface_name:None,ruleset:Vec::new(),
        };
        filter.init().unwrap();
        
        let mut middle= LinuxPacketProcessor {
            filter_mode:true,packet:None,filter:Vec::new(),filter_action:FilterAction::Pass
        };
        let mut queue =Queue::open().unwrap();
        queue.bind(0).unwrap();
        // queue.unwrap().recv()
    }
}