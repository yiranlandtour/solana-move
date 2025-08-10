module cross_chain_token::token {
    use std::signer;
    use aptos_framework::coin::{Self, Coin};
    use aptos_framework::event;
    
    /// Token 结构体
    struct Token has key {
        supply: u64,
        decimals: u8,
    }
    
    /// 用户余额
    struct Balance has key {
        coins: Coin<Token>,
    }
    
    /// 跨链锁定事件
    struct CrossChainLockEvent has drop, store {
        from: address,
        amount: u64,
        target_chain: u32,
        timestamp: u64,
    }
    
    /// 初始化代币
    public fun initialize(account: &signer, decimals: u8) {
        let token = Token {
            supply: 0,
            decimals,
        };
        move_to(account, token);
    }
    
    /// 铸造代币
    public fun mint(account: &signer, to: address, amount: u64) acquires Token, Balance {
        let token = borrow_global_mut<Token>(@cross_chain_token);
        token.supply = token.supply + amount;
        
        if (!exists<Balance>(to)) {
            move_to(account, Balance { coins: coin::zero<Token>() });
        };
        
        let balance = borrow_global_mut<Balance>(to);
        coin::merge(&mut balance.coins, coin::mint<Token>(amount, account));
    }
    
    /// 转账
    public fun transfer(from: &signer, to: address, amount: u64) acquires Balance {
        let from_addr = signer::address_of(from);
        let from_balance = borrow_global_mut<Balance>(from_addr);
        
        let coins_to_transfer = coin::extract(&mut from_balance.coins, amount);
        
        if (!exists<Balance>(to)) {
            move_to(from, Balance { coins: coin::zero<Token>() });
        };
        
        let to_balance = borrow_global_mut<Balance>(to);
        coin::merge(&mut to_balance.coins, coins_to_transfer);
    }
    
    /// 锁定代币用于跨链
    public fun lock_for_bridge(from: &signer, amount: u64, target_chain: u32) acquires Balance {
        let from_addr = signer::address_of(from);
        let balance = borrow_global_mut<Balance>(from_addr);
        
        let locked_coins = coin::extract(&mut balance.coins, amount);
        coin::burn(locked_coins, from);
        
        event::emit(CrossChainLockEvent {
            from: from_addr,
            amount,
            target_chain,
            timestamp: aptos_framework::timestamp::now_microseconds(),
        });
    }
}