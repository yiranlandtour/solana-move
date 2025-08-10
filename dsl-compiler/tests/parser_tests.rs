use cross_chain_dsl::{Contract, Type, Visibility};

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
    
    let contract = Contract::parse(input).expect("Failed to parse contract");
    
    assert_eq!(contract.name, "SimpleToken");
    assert_eq!(contract.state.len(), 2);
    assert_eq!(contract.functions.len(), 1);
    
    let func = &contract.functions[0];
    assert_eq!(func.name, "initialize");
    assert!(matches!(func.visibility, Visibility::Public));
}

#[test]
fn test_parse_complex_types() {
    let input = r#"
        contract ComplexTypes {
            state {
                balances: map<address, u64>;
                allowances: map<address, map<address, u64>>;
                users: vec<address>;
            }
        }
    "#;
    
    let contract = Contract::parse(input).expect("Failed to parse contract");
    
    assert_eq!(contract.state.len(), 3);
    
    // Check map type
    match &contract.state[0].ty {
        Type::Map(k, v) => {
            assert!(matches!(**k, Type::Address));
            assert!(matches!(**v, Type::U64));
        }
        _ => panic!("Expected map type"),
    }
    
    // Check nested map type
    match &contract.state[1].ty {
        Type::Map(k, v) => {
            assert!(matches!(**k, Type::Address));
            assert!(matches!(**v, Type::Map(_, _)));
        }
        _ => panic!("Expected nested map type"),
    }
    
    // Check vec type
    match &contract.state[2].ty {
        Type::Vec(elem) => {
            assert!(matches!(**elem, Type::Address));
        }
        _ => panic!("Expected vec type"),
    }
}

#[test]
fn test_parse_expressions() {
    let input = r#"
        contract ExpressionTest {
            public fn calculate() -> u64 {
                let a = 10;
                let b = 20;
                let c = a + b * 2;
                return c;
            }
        }
    "#;
    
    let contract = Contract::parse(input).expect("Failed to parse contract");
    
    assert_eq!(contract.functions.len(), 1);
    let func = &contract.functions[0];
    assert_eq!(func.name, "calculate");
    assert!(func.return_type.is_some());
    assert_eq!(func.body.len(), 4);
}

#[test]
fn test_parse_control_flow() {
    let input = r#"
        contract ControlFlow {
            public fn transfer(to: address, amount: u64) {
                if (amount > 0) {
                    require(balances[msg_sender()] >= amount, "Insufficient balance");
                    balances[msg_sender()] = balances[msg_sender()] - amount;
                    balances[to] = balances[to] + amount;
                    emit Transfer(msg_sender(), to, amount);
                } else {
                    revert("Invalid amount");
                }
            }
        }
    "#;
    
    let contract = Contract::parse(input).expect("Failed to parse contract");
    
    assert_eq!(contract.functions.len(), 1);
    let func = &contract.functions[0];
    assert_eq!(func.name, "transfer");
    assert_eq!(func.params.len(), 2);
}

#[test]
fn test_parse_error_invalid_syntax() {
    let input = r#"
        contract InvalidSyntax {
            state {
                balance: unknown_type;
            }
        }
    "#;
    
    let result = Contract::parse(input);
    assert!(result.is_err());
}

#[test]
fn test_parse_full_token_contract() {
    let input = std::fs::read_to_string("examples/token.ccdsl")
        .expect("Failed to read token example");
    
    let contract = Contract::parse(&input).expect("Failed to parse token contract");
    
    assert_eq!(contract.name, "Token");
    assert!(contract.state.len() > 0);
    assert!(contract.functions.len() > 0);
    
    // Verify key functions exist
    let function_names: Vec<_> = contract.functions.iter()
        .map(|f| f.name.as_str())
        .collect();
    
    assert!(function_names.contains(&"initialize"));
    assert!(function_names.contains(&"transfer"));
    assert!(function_names.contains(&"mint"));
    assert!(function_names.contains(&"burn"));
}