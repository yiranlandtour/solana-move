use cross_chain_dsl::{Contract, semantic::SemanticAnalyzer};

#[test]
fn test_semantic_valid_contract() {
    let input = r#"
        contract ValidContract {
            state {
                balance: u64;
                owner: address;
            }
            
            public fn initialize() {
                balance = 0;
                owner = msg_sender();
            }
            
            public fn add(amount: u64) {
                balance = balance + amount;
            }
        }
    "#;
    
    let contract = Contract::parse(input).expect("Failed to parse");
    let mut analyzer = SemanticAnalyzer::new();
    
    let result = analyzer.analyze(&contract);
    assert!(result.is_ok());
}

#[test]
fn test_semantic_undefined_variable() {
    let input = r#"
        contract InvalidContract {
            public fn test() {
                undefined_var = 10;
            }
        }
    "#;
    
    let contract = Contract::parse(input).expect("Failed to parse");
    let mut analyzer = SemanticAnalyzer::new();
    
    let result = analyzer.analyze(&contract);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unknown symbol"));
}

#[test]
fn test_semantic_type_mismatch() {
    let input = r#"
        contract TypeMismatch {
            state {
                count: u64;
            }
            
            public fn test() {
                count = true;  // Type error: bool to u64
            }
        }
    "#;
    
    let contract = Contract::parse(input).expect("Failed to parse");
    let mut analyzer = SemanticAnalyzer::new();
    
    let result = analyzer.analyze(&contract);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Type mismatch"));
}

#[test]
fn test_semantic_duplicate_function() {
    let input = r#"
        contract DuplicateFunction {
            public fn test() {
                let a = 1;
            }
            
            public fn test() {  // Duplicate
                let b = 2;
            }
        }
    "#;
    
    let contract = Contract::parse(input).expect("Failed to parse");
    let mut analyzer = SemanticAnalyzer::new();
    
    let result = analyzer.analyze(&contract);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Duplicate function"));
}

#[test]
fn test_semantic_scope_analysis() {
    let input = r#"
        contract ScopeTest {
            public fn test() {
                let x = 10;
                if (x > 5) {
                    let y = 20;
                    let z = x + y;  // Valid: x is in outer scope
                }
                // y is not accessible here
                let a = x;  // Valid
            }
        }
    "#;
    
    let contract = Contract::parse(input).expect("Failed to parse");
    let mut analyzer = SemanticAnalyzer::new();
    
    let result = analyzer.analyze(&contract);
    assert!(result.is_ok());
}

#[test]
fn test_semantic_require_condition_type() {
    let input = r#"
        contract RequireTest {
            public fn test() {
                require(10, "Not a boolean");  // Error: require needs bool
            }
        }
    "#;
    
    let contract = Contract::parse(input).expect("Failed to parse");
    let mut analyzer = SemanticAnalyzer::new();
    
    let result = analyzer.analyze(&contract);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("must be boolean"));
}

#[test]
fn test_semantic_function_call_validation() {
    let input = r#"
        contract FunctionCallTest {
            public fn add(a: u64, b: u64) -> u64 {
                return a + b;
            }
            
            public fn test() {
                let result = add(10, 20);  // Valid
                let error = add(10);       // Error: wrong arg count
            }
        }
    "#;
    
    let contract = Contract::parse(input).expect("Failed to parse");
    let mut analyzer = SemanticAnalyzer::new();
    
    let result = analyzer.analyze(&contract);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("expects 2 arguments"));
}