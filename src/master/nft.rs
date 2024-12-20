#[cfg(target_os = "linux")]
use nftables::{
    batch::Batch,
    expr::{Expression, NamedExpression, Meta, MetaKey, Payload, PayloadField},
    helper::apply_ruleset,
    schema::{Chain, NfListObject, NfObject, Rule, Table},
    stmt::{Match, Operator, Queue, Statement},
    types::{NfChainPolicy, NfChainType, NfFamily, NfHook},
};
#[cfg(target_os = "linux")]
use crate::globals::{QUEUE_NUM,get_interface_name};
#[cfg(target_os = "linux")]
fn create_nftables_objects() -> Vec<NfObject> {
    // 创建 IPv6 表和链
    let table = Table {
        family: NfFamily::IP6,
        name: "rafilter".to_string(),
        handle: None,
    };
    let chain = Chain {
        family: NfFamily::IP6,
        table: table.name.clone(),
        name: "input".to_string(),
        _type: Some(NfChainType::Filter),
        hook: Some(NfHook::Input),
        prio: Some(0),
        policy: Some(NfChainPolicy::Accept),
        ..Default::default()
    };

     let mut rule_expr=vec![
        Statement::Match(Match {
            left: Expression::Named(NamedExpression::Payload(Payload::PayloadField(
                PayloadField {
                    protocol: "icmpv6".to_string(),
                    field: "type".to_string(),
                },
            ))),
            right: Expression::Number(134), // ICMPv6 Router Advertisement
            op: Operator::EQ,
        }),
        Statement::Queue(Queue {
            num: Expression::Number(QUEUE_NUM as u32),
            flags: None,
        }),
     ];
    let interface_name= get_interface_name();
    if let Some(the_name)= interface_name {
        rule_expr.insert(0,
            Statement::Match(Match {
                left:Expression::Named(NamedExpression::Meta(Meta{key:MetaKey::Iifname})),
                right: Expression::String(the_name.name),
                op: Operator::EQ,
            }),
        );
    }
    // 创建匹配 ICMPv6 Router Advertisement 的规则
    let rule = Rule {
        family: NfFamily::IP6,
        table: table.name.clone(),
        chain: chain.name.clone(),
        expr: rule_expr,
        comment: Some("Queue ICMPv6 Router Advertisement packets".to_string()),
        ..Default::default()
    };
    
    vec![
        NfObject::ListObject(Box::new(NfListObject::Table(table))),
        NfObject::ListObject(Box::new(NfListObject::Chain(chain))),
        NfObject::ListObject(Box::new(NfListObject::Rule (rule ))),
    ]
}

// 执行多个 nftables 操作命令
#[cfg(target_os = "linux")]
fn apply_nftables_action(action:Action) -> Result<(), Box<dyn std::error::Error>> {

    let ruleset = create_nftables_objects();
    let mut batch = Batch::new();

    match action {
        Action::AddAll => batch.add_all(ruleset),
        Action::DeleteAll => {
            for obj in ruleset.iter() {
                // 对 NfObject::ListObject 解构并处理
                if let NfObject::ListObject(list_obj) = obj {
                    match list_obj.as_ref() {
                        NfListObject::Table(_) => 
                            batch.delete(*list_obj.clone()),
                        _ => {}, // 对于非表对象，不执行任何操作
                    }
                } else {
                    eprintln!("Unexpected NfObject variant");
                }
            }
        },
        //_ => return Err("Invalid action".into()),
    }
// 执行 nftables 命令
    apply_ruleset(&batch.to_nftables(), None, None)?;

    Ok(())
}
#[cfg(target_os = "linux")]
enum Action {
    AddAll,
    DeleteAll,
}
#[cfg(target_os = "linux")]
pub fn setup_nftables() -> Result<(), Box<dyn std::error::Error>> {
    apply_nftables_action(Action::AddAll)?;

    Ok(())
}
#[cfg(target_os = "linux")]
pub fn delete_nftables() -> Result<(), Box<dyn std::error::Error>> {
    apply_nftables_action(Action::DeleteAll)?;

    Ok(())
}
    // 将所有命令对象放入 nftables 对象中
    // let nftables = Nftables {
    //     objects: actions.into_iter().map(NfObject::CmdObject).collect(),
    // };

            // expr: vec![
        //     Statement::Match(Match {
        //         left:Expression::Named(NamedExpression::Meta(Meta{key:MetaKey::Iifname})),
        //         right: Expression::String(thename),
        //         op: Operator::EQ,
        //     }),
        //     Statement::Match(Match {
        //         left: Expression::Named(NamedExpression::Payload(Payload::PayloadField(
        //             PayloadField {
        //                 protocol: "icmpv6".to_string(),
        //                 field: "type".to_string(),
        //             },
        //         ))),
        //         right: Expression::Number(134), // ICMPv6 Router Advertisement
        //         op: Operator::EQ,
        //     }),
        //     Statement::Queue(Queue {
        //         num: Expression::Number(QUEUE_NUM as u32),
        //         flags: None,
        //     }),
        // ],