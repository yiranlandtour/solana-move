use dsl_compiler::{Contract, DslParser, Rule};
use pest::Parser;

#[test]
fn test_parse_simple_contract() {
    let input = r#"
contract SimpleToken {
    state {
        total_supply: u64;
        owner: address;
    }
    
    public fn initialize(supply: u64) {
        total_supply = supply;
        owner = msg_sender();
    }
}
"#;

    let result = DslParser::parse(Rule::program, input);
    assert!(result.is_ok(), "Failed to parse simple contract");
    
    let pairs = result.unwrap();
    assert!(pairs.len() > 0, "No parse tree generated");
}

#[test]
fn test_parse_complex_contract() {
    let input = r#"
contract AMM {
    state {
        pools: map<address, Pool>;
        fee_rate: u64;
        admin: address;
    }
    
    struct Pool {
        token_a: address;
        token_b: address;
        reserve_a: u64;
        reserve_b: u64;
    }
    
    public fn swap(pool_id: address, amount_in: u64) -> u64 {
        require(pools[pool_id] != null, "Pool not found");
        let pool = pools[pool_id];
        
        // Calculate output amount
        let amount_out = calculate_output(amount_in, pool.reserve_a, pool.reserve_b);
        
        // Update reserves
        pool.reserve_a = pool.reserve_a + amount_in;
        pool.reserve_b = pool.reserve_b - amount_out;
        
        emit Swap(msg_sender(), amount_in, amount_out);
        return amount_out;
    }
    
    private fn calculate_output(amount_in: u64, reserve_in: u64, reserve_out: u64) -> u64 {
        let amount_with_fee = amount_in * (10000 - fee_rate);
        let numerator = amount_with_fee * reserve_out;
        let denominator = (reserve_in * 10000) + amount_with_fee;
        return numerator / denominator;
    }
}
"#;

    let result = DslParser::parse(Rule::program, input);
    assert!(result.is_ok(), "Failed to parse complex contract");
}

#[test]
fn test_parse_with_events() {
    let input = r#"
contract EventContract {
    event Transfer(from: address, to: address, amount: u64);
    event Approval(owner: address, spender: address, amount: u64);
    
    state {
        balances: map<address, u64>;
    }
    
    public fn transfer(to: address, amount: u64) {
        let from = msg_sender();
        require(balances[from] >= amount, "Insufficient balance");
        
        balances[from] = balances[from] - amount;
        balances[to] = balances[to] + amount;
        
        emit Transfer(from, to, amount);
    }
}
"#;

    let result = DslParser::parse(Rule::program, input);
    assert!(result.is_ok(), "Failed to parse contract with events");
}

#[test]
fn test_parse_with_modifiers() {
    let input = r#"
contract SecureContract {
    state {
        owner: address;
        paused: bool;
    }
    
    modifier onlyOwner() {
        require(msg_sender() == owner, "Only owner");
        _;
    }
    
    modifier whenNotPaused() {
        require(!paused, "Contract is paused");
        _;
    }
    
    public fn sensitive_operation() onlyOwner whenNotPaused {
        // Do something sensitive
    }
}
"#;

    let result = DslParser::parse(Rule::program, input);
    assert!(result.is_ok(), "Failed to parse contract with modifiers");
}

#[test]
fn test_parse_all_types() {
    let input = r#"
contract TypeTest {
    state {
        uint8_val: u8;
        uint64_val: u64;
        uint128_val: u128;
        bool_val: bool;
        addr_val: address;
        str_val: string;
        map_val: map<address, u64>;
        vec_val: vec<string>;
        opt_val: Option<u64>;
        res_val: Result<u64, string>;
    }
}
"#;

    let result = DslParser::parse(Rule::program, input);
    assert!(result.is_ok(), "Failed to parse contract with all types");
}

#[test]
fn test_parse_control_flow() {
    let input = r#"
contract ControlFlow {
    public fn complex_logic(x: u64) -> u64 {
        let mut result = 0;
        
        if x > 100 {
            result = x * 2;
        } else if x > 50 {
            result = x + 50;
        } else {
            result = x;
        }
        
        while result < 1000 {
            result = result * 2;
        }
        
        for i in 0..10 {
            result = result + i;
        }
        
        match result {
            0 => return 1,
            1..10 => return 2,
            _ => return result
        }
    }
}
"#;

    let result = DslParser::parse(Rule::program, input);
    assert!(result.is_ok(), "Failed to parse contract with control flow");
}

#[test]
fn test_ast_construction() {
    let input = r#"
contract TestContract {
    state {
        value: u64;
    }
    
    public fn get_value() -> u64 {
        return value;
    }
    
    public fn set_value(new_value: u64) {
        value = new_value;
    }
}
"#;

    let contract = Contract::parse(input);
    assert!(contract.is_ok(), "Failed to construct AST");
    
    let contract = contract.unwrap();
    assert_eq!(contract.name, "TestContract");
    assert_eq!(contract.state.len(), 1);
    assert_eq!(contract.functions.len(), 2);
}

#[test]
fn test_error_handling() {
    // Test missing closing brace
    let input = r#"
contract Broken {
    state {
        value: u64;
"#;

    let result = DslParser::parse(Rule::program, input);
    assert!(result.is_err(), "Should fail on syntax error");
}

#[test]
fn test_lambda_expressions() {
    let input = r#"
contract LambdaTest {
    public fn map_values(values: vec<u64>) -> vec<u64> {
        return values.map(|x| x * 2);
    }
    
    public fn filter_values(values: vec<u64>) -> vec<u64> {
        return values.filter(|x| x > 100);
    }
}
"#;

    let result = DslParser::parse(Rule::program, input);
    assert!(result.is_ok(), "Failed to parse lambda expressions");
}

#[test]
fn test_special_identifiers() {
    let input = r#"
contract SpecialIdentifiers {
    public fn get_sender() -> address {
        return msg_sender;
    }
    
    public fn get_value() -> u64 {
        return msg_value;
    }
    
    public fn get_block() -> u64 {
        return block_number;
    }
    
    public fn get_timestamp() -> u64 {
        return block_timestamp;
    }
}
"#;

    let result = DslParser::parse(Rule::program, input);
    assert!(result.is_ok(), "Failed to parse special identifiers");
}