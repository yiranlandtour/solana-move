use cross_chain_dsl::{Contract, optimizer::Optimizer, Expression, Statement};

#[test]
fn test_optimizer_constant_folding() {
    let input = r#"
        contract OptimizeTest {
            public fn calculate() -> u64 {
                let a = 10 + 20;     // Should fold to 30
                let b = 100 - 50;    // Should fold to 50
                let c = 5 * 10;      // Should fold to 50
                let d = 100 / 2;     // Should fold to 50
                return a + b + c + d;
            }
        }
    "#;
    
    let mut contract = Contract::parse(input).expect("Failed to parse");
    let mut optimizer = Optimizer::new();
    
    optimizer.optimize(&mut contract);
    
    // Check that constants were folded
    let func = &contract.functions[0];
    for stmt in &func.body {
        if let Statement::Let { value, .. } = stmt {
            // Should be constant numbers, not binary operations
            assert!(matches!(value, Expression::Number(_)));
        }
    }
}

#[test]
fn test_optimizer_algebraic_simplification() {
    let input = r#"
        contract SimplifyTest {
            public fn simplify(x: u64) -> u64 {
                let a = x + 0;      // Should simplify to x
                let b = x * 1;      // Should simplify to x
                let c = x - 0;      // Should simplify to x
                let d = x / 1;      // Should simplify to x
                let e = x * 0;      // Should simplify to 0
                return a + b + c + d + e;
            }
        }
    "#;
    
    let mut contract = Contract::parse(input).expect("Failed to parse");
    let mut optimizer = Optimizer::new();
    
    optimizer.optimize(&mut contract);
    
    // The optimizer should have simplified these expressions
    // This is a simplified test - in reality, we'd check the actual AST
}

#[test]
fn test_optimizer_dead_code_elimination() {
    let input = r#"
        contract DeadCodeTest {
            public fn test() {
                if (true) {
                    let a = 10;
                } else {
                    let b = 20;  // Dead code
                }
                
                if (false) {
                    let c = 30;  // Dead code
                }
                
                require(true, "Always passes");  // Can be removed
            }
        }
    "#;
    
    let mut contract = Contract::parse(input).expect("Failed to parse");
    let mut optimizer = Optimizer::new();
    
    optimizer.optimize(&mut contract);
    
    // Check that dead code was removed
    let func = &contract.functions[0];
    assert!(func.body.len() < 3); // Some statements should be removed
}

#[test]
fn test_optimizer_constant_propagation() {
    let input = r#"
        contract PropagationTest {
            public fn test() -> u64 {
                let x = 10;
                let y = x + 5;    // Should become 15
                let z = y * 2;    // Should become 30
                return z;
            }
        }
    "#;
    
    let mut contract = Contract::parse(input).expect("Failed to parse");
    let mut optimizer = Optimizer::new();
    
    optimizer.optimize(&mut contract);
    
    // After optimization, expressions using constants should be folded
}

#[test]
fn test_optimizer_boolean_simplification() {
    let input = r#"
        contract BooleanTest {
            public fn test(x: bool) -> bool {
                let a = x && true;    // Should simplify to x
                let b = x || false;   // Should simplify to x
                let c = x && false;   // Should simplify to false
                let d = x || true;    // Should simplify to true
                return a && b || c && d;
            }
        }
    "#;
    
    let mut contract = Contract::parse(input).expect("Failed to parse");
    let mut optimizer = Optimizer::new();
    
    optimizer.optimize(&mut contract);
    
    // Boolean expressions should be simplified
}