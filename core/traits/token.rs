use crate::types::{Address, Result, Error};

/// 统一的代币操作接口
/// 所有链上实现都必须实现这个 trait
pub trait TokenOperations {
    /// 转账操作
    fn transfer(&self, from: Address, to: Address, amount: u64) -> Result<()>;
    
    /// 铸造代币
    fn mint(&self, to: Address, amount: u64) -> Result<()>;
    
    /// 销毁代币
    fn burn(&self, from: Address, amount: u64) -> Result<()>;
    
    /// 查询余额
    fn balance_of(&self, account: Address) -> Result<u64>;
    
    /// 获取总供应量
    fn total_supply(&self) -> Result<u64>;
}

/// 跨链操作接口
pub trait CrossChainOperations {
    /// 锁定资产准备跨链
    fn lock_for_bridge(&self, from: Address, amount: u64, target_chain: u32) -> Result<()>;
    
    /// 从跨链桥接收资产
    fn receive_from_bridge(&self, to: Address, amount: u64, source_chain: u32) -> Result<()>;
    
    /// 验证跨链消息
    fn verify_cross_chain_message(&self, message: &[u8], signature: &[u8]) -> Result<bool>;
}