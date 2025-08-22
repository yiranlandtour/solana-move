#!/usr/bin/env python3
"""
Simple demo showing AI integration capabilities for CrossChain DSL
"""

import asyncio
import os
from pathlib import Path

# Set up paths
import sys
sys.path.append(str(Path(__file__).parent / "ai-integration"))

from ai_assistant import AICodeGenerator, AISecurityAuditor, AIOptimizer

async def demo_code_generation():
    """Demonstrate AI-powered code generation"""
    print("=" * 60)
    print("🤖 AI Code Generation Demo")
    print("=" * 60)
    
    generator = AICodeGenerator()
    
    # Example 1: Generate a simple token contract
    print("\n📝 Generating Token Contract...")
    description = "Create a token contract with mint and burn functions, 18 decimals, pausable"
    
    code = await generator.generate_from_description(description)
    print("\n✅ Generated Code:")
    print(code[:800] + "..." if len(code) > 800 else code)
    
    # Example 2: Generate an AMM contract
    print("\n\n📝 Generating AMM DEX Contract...")
    description = "Create an AMM DEX with swap function, 0.3% fee, liquidity pools"
    
    code = await generator.generate_from_description(description)
    print("\n✅ Generated Code:")
    print(code[:800] + "..." if len(code) > 800 else code)

async def demo_security_audit():
    """Demonstrate AI-powered security auditing"""
    print("\n" + "=" * 60)
    print("🔍 AI Security Audit Demo")
    print("=" * 60)
    
    # Sample vulnerable contract
    vulnerable_code = """
contract VulnerableToken {
    state {
        balances: map<address, u64>;
        total_supply: u64;
        owner: address;
    }
    
    public fn transfer(to: address, amount: u64) {
        // VULNERABILITY: No balance check!
        balances[msg_sender()] = balances[msg_sender()] - amount;
        balances[to] = balances[to] + amount;
    }
    
    public fn mint(to: address, amount: u64) {
        // VULNERABILITY: No access control!
        total_supply = total_supply + amount;
        balances[to] = balances[to] + amount;
    }
}
"""
    
    auditor = AISecurityAuditor()
    issues = await auditor.audit_contract(vulnerable_code)
    
    print("\n🔒 Security Audit Results:")
    print(f"Found {len(issues)} issues\n")
    
    for i, issue in enumerate(issues[:5], 1):  # Show first 5 issues
        print(f"{i}. [{issue.level.value.upper()}] {issue.category}")
        print(f"   {issue.description}")
        if issue.suggestion:
            print(f"   💡 {issue.suggestion}")
        print()

async def demo_optimization():
    """Demonstrate AI-powered optimization"""
    print("=" * 60)
    print("⚡ AI Optimization Demo")
    print("=" * 60)
    
    # Sample unoptimized code
    unoptimized_code = """
contract UnoptimizedContract {
    state {
        data: map<address, map<address, u64>>;
        items: vec<u64>;
    }
    
    public fn process_batch(addresses: vec<address>) {
        for addr in addresses {
            let value = data[addr][msg_sender()];
            data[addr][msg_sender()] = value + 1;
        }
    }
}
"""
    
    optimizer = AIOptimizer()
    suggestions = optimizer.analyze_gas_usage(unoptimized_code)
    
    print("\n⛽ Optimization Suggestions:")
    for i, suggestion in enumerate(suggestions, 1):
        print(f"\n{i}. {suggestion.category}: {suggestion.description}")
        print(f"   Impact: {suggestion.impact}")
        if suggestion.gas_saving:
            print(f"   Estimated savings: ~{suggestion.gas_saving} gas")

def check_environment():
    """Check if the environment is properly configured"""
    print("\n🔧 Checking Environment...")
    
    # Check Python version
    import sys
    python_version = sys.version_info
    print(f"Python version: {python_version.major}.{python_version.minor}.{python_version.micro}")
    
    # Check for required modules
    required_modules = ["ai_assistant"]
    missing = []
    
    for module in required_modules:
        try:
            __import__(module)
            print(f"✅ Module '{module}' found")
        except ImportError:
            missing.append(module)
            print(f"❌ Module '{module}' not found")
    
    if missing:
        print("\n⚠️  Some modules are missing. Please check your installation.")
        return False
    
    print("\n✅ Environment check passed!")
    return True

async def main():
    """Main demo function"""
    print("\n" + "🌟" * 30)
    print("   CrossChain DSL - AI Integration Demo")
    print("🌟" * 30)
    
    if not check_environment():
        print("\nPlease fix the environment issues before running the demo.")
        return
    
    print("\nSelect a demo to run:")
    print("1. AI Code Generation")
    print("2. AI Security Audit")
    print("3. AI Optimization")
    print("4. Run All Demos")
    print("5. Exit")
    
    choice = input("\nEnter your choice (1-5): ").strip()
    
    try:
        if choice == "1":
            await demo_code_generation()
        elif choice == "2":
            await demo_security_audit()
        elif choice == "3":
            await demo_optimization()
        elif choice == "4":
            await demo_code_generation()
            await demo_security_audit()
            await demo_optimization()
        elif choice == "5":
            print("Exiting...")
            return
        else:
            print("Invalid choice. Please run the script again.")
    except Exception as e:
        print(f"\n❌ Error during demo: {e}")
        print("This is expected if AI API keys are not configured.")
        print("\nTo use AI features with real models, set:")
        print("  export OPENAI_API_KEY='your-key'")
        print("  export ANTHROPIC_API_KEY='your-key'")
    
    print("\n" + "=" * 60)
    print("✅ Demo completed!")
    print("=" * 60)

if __name__ == "__main__":
    asyncio.run(main())