#!/usr/bin/env python3
"""
AI-Powered Assistant for CrossChain DSL
Provides intelligent code generation, security auditing, and optimization
"""

import json
import re
import asyncio
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass
from enum import Enum

# For production, replace with actual API calls
# import anthropic
# import openai

class ContractType(Enum):
    TOKEN = "token"
    AMM = "amm"
    LENDING = "lending"
    NFT = "nft"
    DAO = "dao"
    STAKING = "staking"
    ORACLE = "oracle"
    BRIDGE = "bridge"

class SecurityLevel(Enum):
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"

@dataclass
class SecurityIssue:
    level: SecurityLevel
    category: str
    description: str
    line_number: Optional[int]
    suggestion: str
    code_fix: Optional[str]

@dataclass
class OptimizationSuggestion:
    category: str
    description: str
    impact: str
    before_code: str
    after_code: str
    gas_saving: Optional[int]

class AICodeGenerator:
    """AI-powered code generator for CrossChain DSL"""
    
    def __init__(self):
        self.templates = self._load_templates()
        self.patterns = self._load_patterns()
        
    def _load_templates(self) -> Dict[ContractType, str]:
        """Load contract templates"""
        return {
            ContractType.TOKEN: self._token_template(),
            ContractType.AMM: self._amm_template(),
            ContractType.LENDING: self._lending_template(),
            ContractType.NFT: self._nft_template(),
            ContractType.DAO: self._dao_template(),
        }
    
    def _token_template(self) -> str:
        return """contract {name} {{
    state {{
        total_supply: u64;
        decimals: u8;
        name: string;
        symbol: string;
        balances: map<address, u64>;
        allowances: map<address, map<address, u64>>;
        owner: address;
        paused: bool;
    }}
    
    public fn initialize(
        initial_supply: u64,
        token_decimals: u8,
        token_name: string,
        token_symbol: string
    ) {{
        require(!paused, "Already initialized");
        
        total_supply = initial_supply;
        decimals = token_decimals;
        name = token_name;
        symbol = token_symbol;
        owner = msg_sender();
        
        balances[owner] = initial_supply;
        paused = false;
        
        emit TokenInitialized(owner, initial_supply);
    }}
    
    public fn transfer(to: address, amount: u64) {{
        require(!paused, "Contract paused");
        let from = msg_sender();
        require(balances[from] >= amount, "Insufficient balance");
        
        balances[from] = balances[from] - amount;
        balances[to] = balances[to] + amount;
        
        emit Transfer(from, to, amount);
    }}
    
    {additional_functions}
}}"""

    def _amm_template(self) -> str:
        return """contract {name} {{
    state {{
        pools: map<address, Pool>;
        fee_numerator: u64;
        fee_denominator: u64;
        admin: address;
    }}
    
    struct Pool {{
        token_a: address;
        token_b: address;
        reserve_a: u64;
        reserve_b: u64;
        total_liquidity: u64;
    }}
    
    {functions}
}}"""

    def _lending_template(self) -> str:
        return """contract {name} {{
    state {{
        markets: map<address, Market>;
        user_accounts: map<address, map<address, Account>>;
        oracle: address;
        admin: address;
    }}
    
    struct Market {{
        asset: address;
        total_supply: u64;
        total_borrows: u64;
        borrow_rate: u64;
        supply_rate: u64;
    }}
    
    {functions}
}}"""

    def _nft_template(self) -> str:
        return """contract {name} {{
    state {{
        tokens: map<u64, TokenMetadata>;
        owners: map<u64, address>;
        balances: map<address, u64>;
        next_token_id: u64;
        base_uri: string;
    }}
    
    struct TokenMetadata {{
        name: string;
        description: string;
        uri: string;
        attributes: map<string, string>;
    }}
    
    {functions}
}}"""

    def _dao_template(self) -> str:
        return """contract {name} {{
    state {{
        proposals: map<u64, Proposal>;
        votes: map<u64, map<address, bool>>;
        members: map<address, u64>;
        quorum: u64;
        voting_period: u64;
    }}
    
    struct Proposal {{
        id: u64;
        proposer: address;
        description: string;
        for_votes: u64;
        against_votes: u64;
        end_time: u64;
        executed: bool;
    }}
    
    {functions}
}}"""

    def _load_patterns(self) -> Dict[str, str]:
        """Load common coding patterns"""
        return {
            "reentrancy_guard": """
        require(!locked, "Reentrancy detected");
        locked = true;
        // ... function logic ...
        locked = false;""",
            
            "access_control": """
        require(msg_sender() == admin, "Only admin");""",
            
            "pausable": """
        require(!paused, "Contract is paused");""",
            
            "slippage_protection": """
        require(amount_out >= min_amount_out, "Slippage exceeded");""",
        }

    async def generate_from_description(self, description: str) -> str:
        """Generate DSL code from natural language description"""
        
        # Analyze intent
        contract_type = self._detect_contract_type(description)
        features = self._extract_features(description)
        
        # Select template
        template = self.templates.get(contract_type, self.templates[ContractType.TOKEN])
        
        # Generate custom functions based on features
        custom_functions = self._generate_functions(features)
        
        # Fill template
        code = template.format(
            name=self._extract_name(description),
            additional_functions=custom_functions,
            functions=custom_functions
        )
        
        # Apply security patterns
        code = self._apply_security_patterns(code, features)
        
        return code
    
    def _detect_contract_type(self, description: str) -> ContractType:
        """Detect contract type from description"""
        description_lower = description.lower()
        
        if any(word in description_lower for word in ["amm", "swap", "liquidity", "dex"]):
            return ContractType.AMM
        elif any(word in description_lower for word in ["lend", "borrow", "collateral"]):
            return ContractType.LENDING
        elif any(word in description_lower for word in ["nft", "collectible", "art"]):
            return ContractType.NFT
        elif any(word in description_lower for word in ["dao", "governance", "vote"]):
            return ContractType.DAO
        elif any(word in description_lower for word in ["stake", "staking", "reward"]):
            return ContractType.STAKING
        else:
            return ContractType.TOKEN
    
    def _extract_features(self, description: str) -> List[str]:
        """Extract required features from description"""
        features = []
        description_lower = description.lower()
        
        if "pausable" in description_lower:
            features.append("pausable")
        if "mintable" in description_lower or "mint" in description_lower:
            features.append("mintable")
        if "burnable" in description_lower or "burn" in description_lower:
            features.append("burnable")
        if "fee" in description_lower:
            features.append("fee")
        if "oracle" in description_lower:
            features.append("oracle")
        if "flash" in description_lower:
            features.append("flash_loan")
        
        return features
    
    def _extract_name(self, description: str) -> str:
        """Extract contract name from description"""
        # Simple extraction - in production, use NLP
        words = description.split()
        for i, word in enumerate(words):
            if word.lower() in ["contract", "token", "protocol"]:
                if i > 0:
                    return words[i-1]
        return "MyContract"
    
    def _generate_functions(self, features: List[str]) -> str:
        """Generate functions based on features"""
        functions = []
        
        if "mintable" in features:
            functions.append("""
    public fn mint(to: address, amount: u64) {
        require(msg_sender() == owner, "Only owner can mint");
        total_supply = total_supply + amount;
        balances[to] = balances[to] + amount;
        emit Mint(to, amount);
    }""")
        
        if "burnable" in features:
            functions.append("""
    public fn burn(amount: u64) {
        let from = msg_sender();
        require(balances[from] >= amount, "Insufficient balance");
        balances[from] = balances[from] - amount;
        total_supply = total_supply - amount;
        emit Burn(from, amount);
    }""")
        
        if "pausable" in features:
            functions.append("""
    public fn pause() {
        require(msg_sender() == owner, "Only owner");
        paused = true;
        emit Paused();
    }
    
    public fn unpause() {
        require(msg_sender() == owner, "Only owner");
        paused = false;
        emit Unpaused();
    }""")
        
        return "\n".join(functions)
    
    def _apply_security_patterns(self, code: str, features: List[str]) -> str:
        """Apply security patterns to generated code"""
        if "flash_loan" in features:
            code = code.replace("// ... function logic ...", 
                              self.patterns["reentrancy_guard"])
        return code


class AISecurityAuditor:
    """AI-powered security auditor for smart contracts"""
    
    def __init__(self):
        self.vulnerability_patterns = self._load_vulnerability_patterns()
        
    def _load_vulnerability_patterns(self) -> Dict[str, re.Pattern]:
        """Load regex patterns for vulnerability detection"""
        return {
            "reentrancy": re.compile(r"(transfer|call|send).*\n.*state\s*="),
            "integer_overflow": re.compile(r"(\w+)\s*\+\s*(\w+)(?!\s*<=)"),
            "unchecked_call": re.compile(r"call\([^)]*\)(?!.*require)"),
            "timestamp_dependency": re.compile(r"(timestamp|now|block\.timestamp)"),
            "tx_origin": re.compile(r"tx\.origin"),
            "uninitialized_storage": re.compile(r"let\s+\w+;"),
            "missing_access_control": re.compile(r"public\s+fn\s+\w+\([^)]*\)(?!.*require.*owner)"),
        }
    
    async def audit_contract(self, code: str) -> List[SecurityIssue]:
        """Perform security audit on contract code"""
        issues = []
        
        # Static analysis
        issues.extend(self._static_analysis(code))
        
        # Pattern-based detection
        issues.extend(self._pattern_detection(code))
        
        # Business logic analysis
        issues.extend(self._business_logic_analysis(code))
        
        # Best practices check
        issues.extend(self._best_practices_check(code))
        
        return sorted(issues, key=lambda x: self._severity_score(x.level), reverse=True)
    
    def _static_analysis(self, code: str) -> List[SecurityIssue]:
        """Perform static code analysis"""
        issues = []
        lines = code.split('\n')
        
        for i, line in enumerate(lines, 1):
            # Check for hardcoded values
            if re.search(r'\d{10,}', line) and 'const' not in line:
                issues.append(SecurityIssue(
                    level=SecurityLevel.LOW,
                    category="Hardcoded Value",
                    description="Large hardcoded number detected",
                    line_number=i,
                    suggestion="Consider using constants for large numbers",
                    code_fix=f"const VALUE = {re.search(r'\d{10,}', line).group()};"
                ))
            
            # Check for missing error messages
            if 'require(' in line and ',' not in line:
                issues.append(SecurityIssue(
                    level=SecurityLevel.LOW,
                    category="Missing Error Message",
                    description="Require statement without error message",
                    line_number=i,
                    suggestion="Add descriptive error message to require statement",
                    code_fix=line.replace(')', ', "Error description")')
                ))
        
        return issues
    
    def _pattern_detection(self, code: str) -> List[SecurityIssue]:
        """Detect vulnerabilities using patterns"""
        issues = []
        
        for vuln_name, pattern in self.vulnerability_patterns.items():
            matches = pattern.finditer(code)
            for match in matches:
                line_num = code[:match.start()].count('\n') + 1
                
                if vuln_name == "reentrancy":
                    issues.append(SecurityIssue(
                        level=SecurityLevel.CRITICAL,
                        category="Reentrancy",
                        description="Potential reentrancy vulnerability detected",
                        line_number=line_num,
                        suggestion="Use checks-effects-interactions pattern",
                        code_fix="// Update state before external call"
                    ))
                
                elif vuln_name == "integer_overflow":
                    issues.append(SecurityIssue(
                        level=SecurityLevel.HIGH,
                        category="Integer Overflow",
                        description="Potential integer overflow without bounds check",
                        line_number=line_num,
                        suggestion="Use safe math operations or add overflow checks",
                        code_fix="require(a + b >= a, 'Overflow');"
                    ))
                
                elif vuln_name == "missing_access_control":
                    issues.append(SecurityIssue(
                        level=SecurityLevel.HIGH,
                        category="Access Control",
                        description="Public function without access control",
                        line_number=line_num,
                        suggestion="Add access control modifier",
                        code_fix="require(msg_sender() == admin, 'Only admin');"
                    ))
        
        return issues
    
    def _business_logic_analysis(self, code: str) -> List[SecurityIssue]:
        """Analyze business logic for vulnerabilities"""
        issues = []
        
        # Check for flash loan vulnerabilities
        if "flash" in code.lower() and "fee" not in code.lower():
            issues.append(SecurityIssue(
                level=SecurityLevel.MEDIUM,
                category="Flash Loan",
                description="Flash loan without fee mechanism",
                line_number=None,
                suggestion="Implement flash loan fees to prevent attacks",
                code_fix="let fee = (amount * 5) / 10000; // 0.05% fee"
            ))
        
        # Check for price manipulation
        if "oracle" not in code.lower() and "price" in code.lower():
            issues.append(SecurityIssue(
                level=SecurityLevel.HIGH,
                category="Price Manipulation",
                description="Price calculation without oracle",
                line_number=None,
                suggestion="Use price oracle for accurate pricing",
                code_fix="let price = get_price_from_oracle(asset);"
            ))
        
        return issues
    
    def _best_practices_check(self, code: str) -> List[SecurityIssue]:
        """Check for best practices"""
        issues = []
        
        # Check for events
        if "emit" not in code:
            issues.append(SecurityIssue(
                level=SecurityLevel.LOW,
                category="Best Practice",
                description="No events emitted",
                line_number=None,
                suggestion="Emit events for important state changes",
                code_fix="emit StateChanged(old_value, new_value);"
            ))
        
        # Check for input validation
        if "public fn" in code and "require" not in code:
            issues.append(SecurityIssue(
                level=SecurityLevel.MEDIUM,
                category="Input Validation",
                description="Functions without input validation",
                line_number=None,
                suggestion="Add input validation for all public functions",
                code_fix="require(amount > 0, 'Invalid amount');"
            ))
        
        return issues
    
    def _severity_score(self, level: SecurityLevel) -> int:
        """Get numeric severity score"""
        scores = {
            SecurityLevel.LOW: 1,
            SecurityLevel.MEDIUM: 2,
            SecurityLevel.HIGH: 3,
            SecurityLevel.CRITICAL: 4
        }
        return scores.get(level, 0)
    
    def generate_audit_report(self, issues: List[SecurityIssue]) -> str:
        """Generate formatted audit report"""
        report = """
# Security Audit Report
========================

## Summary
- Total Issues: {}
- Critical: {}
- High: {}
- Medium: {}
- Low: {}

## Detailed Findings
""".format(
            len(issues),
            sum(1 for i in issues if i.level == SecurityLevel.CRITICAL),
            sum(1 for i in issues if i.level == SecurityLevel.HIGH),
            sum(1 for i in issues if i.level == SecurityLevel.MEDIUM),
            sum(1 for i in issues if i.level == SecurityLevel.LOW)
        )
        
        for i, issue in enumerate(issues, 1):
            report += f"""
### {i}. [{issue.level.value.upper()}] {issue.category}
**Description:** {issue.description}
**Location:** Line {issue.line_number if issue.line_number else 'N/A'}
**Recommendation:** {issue.suggestion}
**Suggested Fix:**
```
{issue.code_fix if issue.code_fix else 'N/A'}
```
"""
        
        return report


class AIOptimizer:
    """AI-powered code optimizer"""
    
    def analyze_gas_usage(self, code: str) -> List[OptimizationSuggestion]:
        """Analyze and suggest gas optimizations"""
        suggestions = []
        
        # Storage optimization
        if "map<address, map<" in code:
            suggestions.append(OptimizationSuggestion(
                category="Storage",
                description="Nested mappings can be optimized",
                impact="High",
                before_code="map<address, map<address, u64>>",
                after_code="map<bytes32, u64> // Use keccak256(addr1, addr2) as key",
                gas_saving=2000
            ))
        
        # Loop optimization
        if re.search(r"for.*in.*\{[\s\S]*state\s*=", code):
            suggestions.append(OptimizationSuggestion(
                category="Loop",
                description="State changes in loop can be batched",
                impact="Medium",
                before_code="for item in items { state = state + item; }",
                after_code="let sum = 0; for item in items { sum += item; } state = sum;",
                gas_saving=500
            ))
        
        return suggestions


# CLI Interface
async def main():
    """Main CLI interface for AI assistant"""
    import sys
    
    if len(sys.argv) < 2:
        print("Usage: python ai_assistant.py <command> [options]")
        print("Commands:")
        print("  generate <description> - Generate code from description")
        print("  audit <file> - Audit contract for security issues")
        print("  optimize <file> - Suggest optimizations")
        return
    
    command = sys.argv[1]
    
    if command == "generate":
        if len(sys.argv) < 3:
            print("Please provide a description")
            return
        
        description = " ".join(sys.argv[2:])
        generator = AICodeGenerator()
        code = await generator.generate_from_description(description)
        print(code)
    
    elif command == "audit":
        if len(sys.argv) < 3:
            print("Please provide a file path")
            return
        
        with open(sys.argv[2], 'r') as f:
            code = f.read()
        
        auditor = AISecurityAuditor()
        issues = await auditor.audit_contract(code)
        report = auditor.generate_audit_report(issues)
        print(report)
    
    elif command == "optimize":
        if len(sys.argv) < 3:
            print("Please provide a file path")
            return
        
        with open(sys.argv[2], 'r') as f:
            code = f.read()
        
        optimizer = AIOptimizer()
        suggestions = optimizer.analyze_gas_usage(code)
        
        print("# Optimization Suggestions\n")
        for i, suggestion in enumerate(suggestions, 1):
            print(f"{i}. {suggestion.category}: {suggestion.description}")
            print(f"   Impact: {suggestion.impact}")
            print(f"   Gas Saving: ~{suggestion.gas_saving} units")
            print()


if __name__ == "__main__":
    asyncio.run(main())