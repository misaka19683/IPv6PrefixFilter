use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use std::sync::atomic::Ordering;
use std::thread;
use ipnet::Ipv6Net;
// use log::info;
use nftables::batch::Batch;
use nftables::helper;
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
        helper::apply_ruleset(&batch.to_nftables()).unwrap();

        Ok(())
    }
}
impl LinuxFilter<'_> {
    pub fn new(queue_num: u32, interface_name: Option<NetworkInterface>) -> Self {
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
use crate::RUNNING;

#[derive(Debug, PartialEq)]
pub struct LinuxPacketProcessor {
    pub(crate) filter_mode:FilterMode,
    pub(crate) filter:Vec<Ipv6Net>,
}

impl PacketProcessor for LinuxPacketProcessor {
    fn filter(&self) -> &Vec<Ipv6Net> { &self.filter }

    fn filter_mode(&self) -> FilterMode { self.filter_mode }

    fn run(&mut self) -> Result<(), FilterError> {
        let mut queue = Queue::open().map_err(|e| FilterError::InitError(e.to_string()))?;
        queue.bind(0).unwrap();

        queue.set_nonblocking(true);
        queue.set_fail_open(0, false).unwrap(); //非阻塞+无法接受的包丢弃

        while RUNNING.load(Ordering::SeqCst) {
            let message =queue.recv();
            match message { 
                Err(_)=>thread::sleep(std::time::Duration::from_millis(50)),
                Ok(mut msg)=>{
                    if let Ok(result)=self.analyze_packet(msg.get_payload()) {
                        msg.set_verdict(if result { Verdict::Accept } else { Verdict::Drop });
                        queue.verdict(msg).unwrap();
                    }
                },
            }
        }

        Ok(())
    }
}

