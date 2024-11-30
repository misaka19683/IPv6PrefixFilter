use nftables::{
    batch::Batch,
    expr::{Payload,PayloadField,NamedExpression,Expression}, 
    helper::apply_ruleset, 
    schema::{Table,Chain,Rule,NfObject,NfListObject,}, 
    stmt::{Statement,Match,Operator,Queue}, 
    types::{NfFamily, NfChainType, NfChainPolicy, NfHook},
};

fn create_nftables_objects() -> Vec<NfObject> {
    // 创建 IPv6 表和链
    let table = Table { family: NfFamily::IP6, name: "rafilter".to_string(), handle: None };
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

    // 创建匹配 ICMPv6 Router Advertisement 的规则
    let rule = Rule {
        family: NfFamily::IP6,
        table: table.name.clone(),
        chain: chain.name.clone(),
        expr: vec![
            Statement::Match(Match {
                left: Expression::Named(NamedExpression::Payload(Payload::PayloadField(PayloadField { protocol: "icmpv6".to_string(), field: "type".to_string() }))),
                right: Expression::Number(134),  // ICMPv6 Router Advertisement
                op: Operator::EQ,
            }),
            Statement::Queue(Queue { num: Expression::Number(0), flags: None }),
        ],
        comment: Some("Queue ICMPv6 Router Advertisement packets".to_string()),
        ..Default::default()
    };

    let (a,b,c) = (
        NfObject::ListObject(Box::new(NfListObject::Table(table))), 
        NfObject::ListObject(Box::new(NfListObject::Chain(chain))), 
        NfObject::ListObject(Box::new(NfListObject::Rule (rule ))),
    );
    return  vec![a,b,c];
}

// 执行多个 nftables 操作命令
fn apply_nftables_action(a:usize) -> Result<(), Box<dyn std::error::Error>> {
    // 将所有命令对象放入 nftables 对象中
    // let nftables = Nftables {
    //     objects: actions.into_iter().map(NfObject::CmdObject).collect(),
    // };

    let ruleset = create_nftables_objects();
    let mut batch = Batch::new();
    if a==1 {
        batch.add_all(ruleset);
    }else {
        for obj in ruleset.iter() {
            // 对 NfObject::ListObject 解构并处理
            if let NfObject::ListObject(list_obj) = obj {
                match list_obj.as_ref() {
                    NfListObject::Table(_) 
                    | NfListObject::Chain(_) 
                    | NfListObject::Rule(_) => {
                        batch.delete(*list_obj.clone());
                    },
                    _ => return Err("Unexpected object type in ruleset".into()),
                }
            } else {
                return Err("Unexpected NfObject variant".into());
            }
        }
    };
    
    let ruleset=batch.to_nftables();

    apply_ruleset(&ruleset,None, None)?;

    Ok(())

}
pub fn setup_nftables() -> Result<(), Box<dyn std::error::Error>> {

    apply_nftables_action(1)?;
    
    Ok(())
}

pub fn delete_nftables() -> Result<(), Box<dyn std::error::Error>> {

    apply_nftables_action(0)?;

    Ok(())
}