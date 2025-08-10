use std::fmt;

/// 统一的地址类型
/// 可以表示 Solana、Aptos、Sui 的地址
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Address {
    Solana([u8; 32]),
    Aptos([u8; 32]),
    Sui([u8; 32]),
}

impl Address {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Address::Solana(bytes) | Address::Aptos(bytes) | Address::Sui(bytes) => bytes.to_vec(),
        }
    }
    
    pub fn chain_type(&self) -> ChainType {
        match self {
            Address::Solana(_) => ChainType::Solana,
            Address::Aptos(_) => ChainType::Aptos,
            Address::Sui(_) => ChainType::Sui,
        }
    }
}

/// 支持的链类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainType {
    Solana,
    Aptos,
    Sui,
}

impl ChainType {
    pub fn chain_id(&self) -> u32 {
        match self {
            ChainType::Solana => 1,
            ChainType::Aptos => 2,
            ChainType::Sui => 3,
        }
    }
}

/// 统一的错误类型
#[derive(Debug)]
pub enum Error {
    InvalidAddress,
    InsufficientBalance,
    Unauthorized,
    CrossChainError(String),
    ChainSpecific(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InvalidAddress => write!(f, "Invalid address"),
            Error::InsufficientBalance => write!(f, "Insufficient balance"),
            Error::Unauthorized => write!(f, "Unauthorized"),
            Error::CrossChainError(msg) => write!(f, "Cross-chain error: {}", msg),
            Error::ChainSpecific(msg) => write!(f, "Chain-specific error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;