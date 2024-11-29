use nftables::{expr::*, helper::apply_ruleset, schema::*, stmt::*, types::*};

fn create_nftables_objects() -> (NfListObject, NfListObject, NfListObject) {
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

    (NfListObject::Table(table), NfListObject::Chain(chain), NfListObject::Rule(rule))
}

// 执行多个 nftables 操作命令
fn apply_nftables_action(actions: Vec<NfCmd>) -> Result<(), Box<dyn std::error::Error>> {
    // 将所有命令对象放入 nftables 对象中
    let nftables = Nftables {
        objects: actions.into_iter().map(NfObject::CmdObject).collect(),
    };

    // 应用 nftables 配置
    apply_ruleset(&nftables, None, None)?;

    Ok(())
}
pub fn setup_nftables() -> Result<(), Box<dyn std::error::Error>> {
    let (table_obj, chain_obj, nf_rule_obj) = create_nftables_objects();

    // 构造操作命令：添加表、链和规则
    let add_table = NfCmd::Add(table_obj);
    let add_chain = NfCmd::Add(chain_obj);
    let add_rule = NfCmd::Add(nf_rule_obj);

    // 执行添加操作
    apply_nftables_action(vec![add_table, add_chain, add_rule])?;
    
    Ok(())
}

pub fn delete_nftables() -> Result<(), Box<dyn std::error::Error>> {
    let (table_obj, chain_obj, nf_rule_obj) = create_nftables_objects();

    // 构造操作命令：删除表、链和规则
    let delete_table = NfCmd::Delete(table_obj);
    let delete_chain = NfCmd::Delete(chain_obj);
    let delete_rule = NfCmd::Delete(nf_rule_obj);

    // 执行删除操作
    apply_nftables_action(vec![delete_table, delete_chain, delete_rule])?;
    
    Ok(())
}