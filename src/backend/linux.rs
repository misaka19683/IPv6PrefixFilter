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
        
        //说不定你的linux没有nft，快去装一个，也可能是没有queue模块，也快去装一个，找找这两个命令，提醒用户
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
        //说明你试图清除nft规则失败了，也许已经被清除了？提醒用户别清了，用nft list ruleset 看看有没有这条规则

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
        queue.bind(0).unwrap();//也可能说明上一个启动的可以在
        //第一优先级，最有可能说明用户的上一个ipv6_prefix_filter程序没有关闭，提醒用户关闭上一个程序。并清理nft规则
        //第二优先级，说明已经有应用程序监听了queue0 ，提醒用户把使用queue0的程序关闭，同时应当添加指定queue_num的功能，以避开其他应用程序，自动清除nft规则
        //第三优先级，以后可以添加功能允许用户指定queue_num,这样可以同时跑很多个此程序，需同时修改nft规则的table名和或者使用同一个table，其他名称的chain
        queue.set_nonblocking(true);
        queue.set_fail_open(0, false).unwrap(); //非阻塞+无法接受的包丢弃
        //第一优先级，无法想象为什么会出错，不过也许应该提醒用户把nft规则clear了
        // 第二优先级，添加报错后自动清除nft规则
        while RUNNING.load(Ordering::SeqCst) {
            let message =queue.recv();
            match message { 
                Err(_)=>thread::sleep(std::time::Duration::from_millis(50)),
                Ok(mut msg)=>{
                    if let Ok(result)=self.analyze_packet(msg.get_payload()) {
                        msg.set_verdict(if result { Verdict::Accept } else { Verdict::Drop });
                        queue.verdict(msg).unwrap();
                        //我觉得这不该报错。提醒清除nft规则
                        //自动清除nft规则
                    }
                },
            }
        }

        Ok(())
    }
}

