use log::info;
use std::borrow::Cow;
use std::net::Ipv6Addr;
use ipnet::Ipv6Net;
use nfq::{Queue, Verdict};
use pnet::packet::{
    icmpv6::{
        ndp::{NdpOptionTypes::PrefixInformation, RouterAdvertPacket},
        Icmpv6Packet,
        Icmpv6Types::RouterAdvert,
    },
    ipv6::Ipv6Packet,
    Packet,
};
use nftables::{
    batch::Batch,
    expr::{Expression, NamedExpression, Meta, MetaKey, Payload, PayloadField},
    helper::apply_ruleset,
    schema::{Chain, NfListObject, NfObject, Rule, Table},
    stmt::{Match, Operator, Queue as NftQueue, Statement},
    types::{NfChainPolicy, NfChainType, NfFamily, NfHook},
};

use crate::AppState;

// --- nft.rs content ---

fn create_nftables_objects(queue_num: u16, interface_name: Option<String>) -> Vec<NfObject<'static>> {
    let table = Table {
        family: NfFamily::IP6,
        name: Cow::from("rafilter"),
        handle: None,
    };
    let chain = Chain {
        family: NfFamily::IP6,
        table: table.name.clone(),
        name: Cow::from("input"),
        _type: Some(NfChainType::Filter),
        hook: Some(NfHook::Input),
        prio: Some(0),
        policy: Some(NfChainPolicy::Accept),
        ..Default::default()
    };

    let mut rule_expr = vec![
        Statement::Match(Match {
            left: Expression::Named(NamedExpression::Payload(Payload::PayloadField(
                PayloadField {
                    protocol: Cow::from("icmpv6"),
                    field: Cow::from("type"),
                },
            ))),
            right: Expression::Number(134),
            op: Operator::EQ,
        }),
        Statement::Queue(NftQueue {
            num: Expression::Number(queue_num as u32),
            flags: None,
        }),
    ];

    if let Some(the_name) = interface_name {
        rule_expr.insert(0,
            Statement::Match(Match {
                left: Expression::Named(NamedExpression::Meta(Meta { key: MetaKey::Iifname })),
                right: Expression::String(Cow::from(the_name)),
                op: Operator::EQ,
            }),
        );
    }
    let rule = Rule {
        family: NfFamily::IP6,
        table: table.name.clone(),
        chain: chain.name.clone(),
        expr: Cow::from(rule_expr),
        comment: Some(Cow::from("Queue ICMPv6 Router Advertisement packets".to_string())),
        ..Default::default()
    };

    vec![
        NfObject::ListObject(NfListObject::Table(table)),
        NfObject::ListObject(NfListObject::Chain(chain)),
        NfObject::ListObject(NfListObject::Rule(rule)),
    ]
}

pub fn setup_nftables(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    let ruleset = create_nftables_objects(state.queue_num, state.interface.as_ref().map(|i| i.name.clone()));
    let mut batch = Batch::new();
    batch.add_all(ruleset);
    apply_ruleset(&batch.to_nftables())?;
    Ok(())
}

pub fn delete_nftables() -> Result<(), Box<dyn std::error::Error>> {
    let mut batch = Batch::new();
    batch.delete(NfListObject::Table(Table {
        family: NfFamily::IP6,
        name: Cow::from("rafilter"),
        handle: None,
    }));
    apply_ruleset(&batch.to_nftables())?;
    Ok(())
}

// --- queue.rs content ---

pub fn process_queue(state: AppState) {
    let mut queue = match Queue::open() {
        Ok(q) => q,
        Err(err) => {
            eprintln!("Failed to open NFQUEUE: {}", err); // 无法打开 NFQUEUE
            return;
        }
    };
    if let Err(err) = queue.bind(state.queue_num) {
        eprintln!("Failed to bind to queue {}: {}", state.queue_num, err); // 无法绑定到队列
        return;
    }
    if let Err(err) = queue.set_fail_open(state.queue_num, false) {
        eprintln!("Failed to set fail-open behavior: {}", err); // 无法设置 fail-open 行为
        return;
    }

    ctrlc::set_handler(move || {
        println!("Signal received, shutting down gracefully..."); // 接收到信号，正在优雅地关闭程序...
        let _ = delete_nftables();
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    loop {
        match queue.recv() {
            Ok(mut msg) => {
                let data = msg.get_payload();
                let prefixes = extract_prefixes(data);
                
                let verdict = if prefixes.is_empty() {
                    Verdict::Accept
                } else {
                    let mut all_ok = true;
                    for prefix in &prefixes {
                        let is_in_list = state.prefixes.iter().any(|p| p.contains(prefix));
                        let allowed = if state.blacklist_mode { !is_in_list } else { is_in_list };
                        
                        if !allowed {
                            info!("Dropped! prefix {}!", prefix);
                            all_ok = false;
                            break;
                        }
                        info!("Accepted prefix {}!", prefix);
                    }
                    if all_ok { Verdict::Accept } else { Verdict::Drop }
                };

                msg.set_verdict(verdict);
                if let Err(e) = queue.verdict(msg) {
                    eprintln!("Failed to send verdict: {}", e); // 发送判决失败
                }
            }
            Err(e) => {
                eprintln!("Failed to receive packet from NFQUEUE: {}", e); // NFQUEUE 接收数据包失败
                break;
            }
        }
    }
}

fn extract_prefixes(data: &[u8]) -> Vec<Ipv6Net> {
    let mut prefixes = Vec::new();

    // Parse IPv6 header
    let ipv6_packet = match Ipv6Packet::new(data) {
        Some(packet) => packet,
        None => return prefixes,
    };

    // Parse ICMPv6 header from IPv6 payload
    let icmp6_packet = match Icmpv6Packet::new(ipv6_packet.payload()) {
        Some(packet) => packet,
        None => return prefixes,
    };

    // We only care about Router Advertisement (RA) packets
    if icmp6_packet.get_icmpv6_type() != RouterAdvert {
        return prefixes;
    }

    // Parse Router Advertisement packet
    let ra_packet = match RouterAdvertPacket::new(icmp6_packet.packet()) {
        Some(packet) => packet,
        None => return prefixes,
    };

    // Iterate through all NDP options in the RA packet
    for op in ra_packet.get_options() {
        // We only care about Prefix Information options
        if op.option_type != PrefixInformation {
            continue;
        }

        // Prefix Information Option (Type 3) structure:
        // Offset 0: Type (3)
        // Offset 1: Length (4, in units of 8 octets)
        // Offset 2: Prefix Length (1 octet)
        // ...
        // Offset 16: Prefix (16 octets)
        //
        // pnet's NdpOption.data starts AFTER the Type and Length fields (2 bytes).
        // So in op.data:
        // Offset 0 (original 2): Prefix Length
        // Offset 14 (original 16): Prefix
        if op.data.len() >= 30 {
            let prefix_len = op.data[0];
            let addr_bytes: [u8; 16] = op.data[14..30].try_into().unwrap_or([0u8; 16]);
            let addr = Ipv6Addr::from(addr_bytes);
            
            // Create Ipv6Net from the extracted address and prefix length
            if let Ok(net) = Ipv6Net::new(addr, prefix_len) {
                prefixes.push(net);
            }
        }
    }
    prefixes
}
