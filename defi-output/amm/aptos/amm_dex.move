module amm_dex::dex {
    use std::signer;
    use aptos_framework::coin::{Self, Coin};
    use aptos_framework::event;
    use aptos_framework::timestamp;
    use aptos_std::simple_map::{Self, SimpleMap};
    use aptos_std::math64;
    
    struct DexState has key {
        admin: address,
        fee_numerator: u64,
        fee_denominator: u64,
        protocol_fee_share: u64,
        pools: SimpleMap<address, Pool>,
        pool_count: u64,
        paused: bool,
        protocol_fees: SimpleMap<address, u64>,
    }
    
    struct Pool has store {
        token_a: address,
        token_b: address,
        reserve_a: u64,
        reserve_b: u64,
        k_last: u128,
        block_timestamp_last: u64,
        price_cumulative_a: u128,
        price_cumulative_b: u128,
        total_lp_supply: u64,
    }
    
    struct LPToken<phantom CoinTypeA, phantom CoinTypeB> has key {
        balance: u64,
    }
    
    public entry fun initialize(
        account: &signer,
        fee_numerator: u64,
        fee_denominator: u64,
    ) {
        let sender = signer::address_of(account);
        assert!(!exists<DexState>(@amm_dex), 1);
        
        move_to(account, DexState {
            admin: sender,
            fee_numerator,
            fee_denominator,
            protocol_fee_share: 6,
            pools: simple_map::create(),
            pool_count: 0,
            paused: false,
            protocol_fees: simple_map::create(),
        });
        
        event::emit(DexInitialized {
            admin: sender,
            fee_numerator,
            fee_denominator,
        });
    }
    
    public entry fun create_pool<CoinTypeA, CoinTypeB>(
        account: &signer,
        initial_amount_a: u64,
        initial_amount_b: u64,
    ) acquires DexState {
        let dex = borrow_global_mut<DexState>(@amm_dex);
        assert!(!dex.paused, 2);
        
        let sender = signer::address_of(account);
        let pool_address = generate_pool_address<CoinTypeA, CoinTypeB>();
        
        assert!(!simple_map::contains_key(&dex.pools, &pool_address), 3);
        
        // Withdraw coins from user
        let coins_a = coin::withdraw<CoinTypeA>(account, initial_amount_a);
        let coins_b = coin::withdraw<CoinTypeB>(account, initial_amount_b);
        
        // Create pool
        let pool = Pool {
            token_a: type_info::type_of<CoinTypeA>(),
            token_b: type_info::type_of<CoinTypeB>(),
            reserve_a: initial_amount_a,
            reserve_b: initial_amount_b,
            k_last: (initial_amount_a as u128) * (initial_amount_b as u128),
            block_timestamp_last: timestamp::now_seconds(),
            price_cumulative_a: 0,
            price_cumulative_b: 0,
            total_lp_supply: math64::sqrt(initial_amount_a * initial_amount_b),
        };
        
        simple_map::add(&mut dex.pools, pool_address, pool);
        dex.pool_count = dex.pool_count + 1;
        
        // Mint LP tokens
        move_to(account, LPToken<CoinTypeA, CoinTypeB> {
            balance: pool.total_lp_supply,
        });
        
        event::emit(PoolCreated {
            pool: pool_address,
            initial_amount_a,
            initial_amount_b,
        });
    }
    
    public entry fun swap<CoinTypeIn, CoinTypeOut>(
        account: &signer,
        amount_in: u64,
        min_amount_out: u64,
    ) acquires DexState {
        let dex = borrow_global_mut<DexState>(@amm_dex);
        assert!(!dex.paused, 2);
        
        let pool_address = get_pool_address<CoinTypeIn, CoinTypeOut>();
        let pool = simple_map::borrow_mut(&mut dex.pools, &pool_address);
        
        // Calculate output amount
        let (reserve_in, reserve_out) = get_reserves<CoinTypeIn, CoinTypeOut>(pool);
        let amount_out = calculate_output_amount(
            amount_in,
            reserve_in,
            reserve_out,
            dex.fee_numerator,
            dex.fee_denominator
        );
        
        assert!(amount_out >= min_amount_out, 4);
        assert!(amount_out < reserve_out, 5);
        
        // Execute swap
        let coins_in = coin::withdraw<CoinTypeIn>(account, amount_in);
        deposit_to_pool(pool_address, coins_in);
        
        let coins_out = withdraw_from_pool<CoinTypeOut>(pool_address, amount_out);
        coin::deposit(signer::address_of(account), coins_out);
        
        // Update reserves
        update_reserves<CoinTypeIn, CoinTypeOut>(pool, amount_in, amount_out);
        
        event::emit(SwapExecuted {
            user: signer::address_of(account),
            amount_in,
            amount_out,
        });
    }
}
