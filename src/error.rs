use ipnet::Ipv6Net;

type Ipv6Prefix = Ipv6Net;
#[derive(thiserror::Error, Debug)]
pub enum FilterError {
    #[error("初始化失败: {0}")]
    InitError(String),

    #[error("规则冲突: {0} 已存在")]
    RuleConflict(Ipv6Prefix),//是个好错误，可以给nft的用用。

    #[error("捕获超时")]
    CaptureTimeout,

    #[error("平台不支持: {0}")]
    UnsupportedPlatform(&'static str),
}