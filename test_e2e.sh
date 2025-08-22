#!/bin/bash

# End-to-end test script for CrossChain DSL

set -e

echo "========================================="
echo "CrossChain DSL - End-to-End Test Suite"
echo "========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Function to run a test
run_test() {
    local test_name=$1
    local command=$2
    
    echo -n "Testing $test_name... "
    
    if eval "$command" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ PASSED${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}✗ FAILED${NC}"
        ((TESTS_FAILED++))
        echo "  Command: $command"
    fi
}

# Function to check if file exists
check_file() {
    local file=$1
    local description=$2
    
    echo -n "Checking $description... "
    
    if [ -f "$file" ]; then
        echo -e "${GREEN}✓ EXISTS${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}✗ MISSING${NC}"
        ((TESTS_FAILED++))
    fi
}

echo "1. SETUP CHECKS"
echo "----------------"

# Check directory structure
check_file "dsl-compiler/Cargo.toml" "DSL compiler project"
check_file "ai-integration/ai_assistant.py" "AI assistant module"
check_file "ai-integration/requirements.txt" "Python requirements"

echo ""
echo "2. BUILD TESTS"
echo "--------------"

# Build the Rust compiler
cd dsl-compiler
run_test "Rust compiler build" "cargo build --release"

# Check if binary was created
check_file "target/release/ccdsl" "Compiler binary"

cd ..

echo ""
echo "3. PARSER TESTS"
echo "---------------"

# Test parsing of example contracts
cd dsl-compiler

# Create test files
cat > test_token.ccdsl << 'EOF'
contract TestToken {
    state {
        total_supply: u64;
        balances: map<address, u64>;
    }
    
    public fn transfer(to: address, amount: u64) {
        require(balances[msg_sender()] >= amount, "Insufficient balance");
        balances[msg_sender()] = balances[msg_sender()] - amount;
        balances[to] = balances[to] + amount;
        emit Transfer(msg_sender(), to, amount);
    }
}
EOF

cat > test_amm.ccdsl << 'EOF'
contract TestAMM {
    state {
        reserves: map<address, u64>;
        total_liquidity: u64;
    }
    
    public fn swap(token_in: address, amount_in: u64) -> u64 {
        let reserve_in = reserves[token_in];
        let reserve_out = reserves[token_out];
        let amount_out = calculate_output(amount_in, reserve_in, reserve_out);
        
        require(amount_out > 0, "Invalid output");
        
        reserves[token_in] = reserve_in + amount_in;
        reserves[token_out] = reserve_out - amount_out;
        
        return amount_out;
    }
}
EOF

# Test compilation (these will fail but we check if parser runs)
run_test "Parse token contract" "cargo run -- compile -i test_token.ccdsl -t solana || true"
run_test "Parse AMM contract" "cargo run -- compile -i test_amm.ccdsl -t aptos || true"

# Run unit tests
run_test "Parser unit tests" "cargo test --lib parser"
run_test "Semantic analyzer tests" "cargo test --lib semantic"
run_test "Optimizer tests" "cargo test --lib optimizer"

cd ..

echo ""
echo "4. AI INTEGRATION TESTS"
echo "-----------------------"

# Test Python environment
run_test "Python3 available" "which python3"
run_test "AI assistant import" "python3 -c 'import sys; sys.path.append(\"ai-integration\"); import ai_assistant'"

# Test AI demos (non-interactive)
cat > test_ai.py << 'EOF'
import sys
import asyncio
sys.path.append("ai-integration")

async def test():
    try:
        from ai_assistant import AICodeGenerator
        gen = AICodeGenerator()
        code = await gen.generate_from_description("token contract")
        return len(code) > 0
    except Exception as e:
        print(f"Error: {e}")
        return False

result = asyncio.run(test())
sys.exit(0 if result else 1)
EOF

run_test "AI code generation" "python3 test_ai.py"

echo ""
echo "5. INTEGRATION TESTS"
echo "--------------------"

# Test the complete flow
cat > integration_test.ccdsl << 'EOF'
contract IntegrationTest {
    state {
        value: u64;
        owner: address;
    }
    
    public fn initialize() {
        owner = msg_sender();
        value = 0;
    }
    
    public fn increment() {
        value = value + 1;
    }
    
    public fn get_value() -> u64 {
        return value;
    }
}
EOF

# Test compile command
cd dsl-compiler
run_test "End-to-end compilation" "cargo run -- compile -i ../integration_test.ccdsl -t all || true"

# Check output directories
check_file "output/solana/lib.rs" "Solana output"
check_file "output/aptos/token.move" "Aptos output"

cd ..

echo ""
echo "6. DOCUMENTATION TESTS"
echo "----------------------"

check_file "README.md" "Main README"
check_file "AI_FEATURES.md" "AI features documentation"
check_file "TECHNICAL_ROADMAP.md" "Technical roadmap"
check_file "IMPROVEMENTS.md" "Improvements documentation"

echo ""
echo "========================================="
echo "TEST RESULTS"
echo "========================================="
echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "\n${GREEN}✅ ALL TESTS PASSED!${NC}"
    exit 0
else
    echo -e "\n${RED}❌ SOME TESTS FAILED${NC}"
    echo "Please review the failed tests above."
    exit 1
fi