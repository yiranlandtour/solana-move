#!/usr/bin/env python3
"""
Demo script showing AI integration capabilities for CrossChain DSL
"""

import asyncio
import os
from pathlib import Path

# Set up paths
import sys
sys.path.append(str(Path(__file__).parent / "ai-integration"))

from ai_config import AIConfigManager, AIProvider, ModelCapability
from ai_assistant import AICodeGenerator, AISecurityAuditor, AIOptimizer

async def demo_ai_code_generation():
    """Demonstrate AI-powered code generation"""
    print("=" * 60)
    print("🤖 AI-Powered Smart Contract Generation Demo")
    print("=" * 60)
    
    # Initialize AI engine
    config = AIConfig()
    engine = AdvancedAIEngine(config)
    
    # Example 1: Generate a DeFi AMM contract
    print("\n📝 Example 1: Generating AMM DEX Contract")
    print("-" * 40)
    
    amm_context = CodeContext(
        contract_type="AMM",
        requirements="""
        Create an automated market maker with:
        - Liquidity pools for token pairs
        - 0.3% swap fee
        - Slippage protection
        - Flash loan support
        - Admin functions for fee adjustment
        """,
        target_chains=["solana", "aptos", "sui"],
        security_level="high",
        optimization_priority="gas"
    )
    
    try:
        result = await engine.generate_smart_contract(amm_context)
        print("\n✅ Generated AMM Contract:")
        print(result.code[:500] + "..." if len(result.code) > 500 else result.code)
        print(f"\n📊 Confidence: {result.confidence:.2%}")
        print(f"⛽ Estimated Gas: {result.estimated_gas}")
        print(f"🔒 Security Score: {result.security_score}")
        
        if result.suggestions:
            print("\n💡 Suggestions:")
            for suggestion in result.suggestions:
                print(f"  - {suggestion}")
    except Exception as e:
        print(f"❌ Generation failed: {e}")
        print("ℹ️  Make sure to set your API keys in environment variables:")
        print("   export OPENAI_API_KEY=your_key_here")
        print("   export ANTHROPIC_API_KEY=your_key_here")
    
    # Example 2: Generate a Lending Protocol
    print("\n\n📝 Example 2: Generating Lending Protocol")
    print("-" * 40)
    
    lending_context = CodeContext(
        contract_type="Lending",
        requirements="""
        Create a lending protocol with:
        - Supply and borrow functions
        - Dynamic interest rates based on utilization
        - Collateral management
        - Liquidation mechanism
        - Oracle price feeds
        """,
        target_chains=["solana", "aptos"],
        security_level="critical",
        optimization_priority="security"
    )
    
    try:
        result = await engine.generate_smart_contract(lending_context)
        print("\n✅ Generated Lending Contract:")
        print(result.code[:500] + "..." if len(result.code) > 500 else result.code)
        print(f"\n📊 Confidence: {result.confidence:.2%}")
        
        if result.warnings:
            print("\n⚠️  Warnings:")
            for warning in result.warnings:
                print(f"  - {warning}")
    except Exception as e:
        print(f"❌ Generation failed: {e}")

async def demo_code_analysis():
    """Demonstrate AI-powered code analysis"""
    print("\n" + "=" * 60)
    print("🔍 AI-Powered Code Analysis Demo")
    print("=" * 60)
    
    # Sample contract to analyze
    sample_code = """
contract VulnerableToken {
    state {
        balances: map<address, u64>;
        total_supply: u64;
    }
    
    public fn transfer(to: address, amount: u64) {
        // Missing balance check!
        balances[msg_sender()] = balances[msg_sender()] - amount;
        balances[to] = balances[to] + amount;
    }
    
    public fn mint(amount: u64) {
        // No access control!
        total_supply = total_supply + amount;
        balances[msg_sender()] = balances[msg_sender()] + amount;
    }
}
"""
    
    config = AIConfig()
    engine = AdvancedAIEngine(config)
    
    print("\n📄 Analyzing contract for security issues...")
    print("-" * 40)
    
    try:
        # Get first available provider
        for provider_type, provider in engine.providers.items():
            if provider:
                analysis = await provider.analyze(sample_code, "security")
                print("\n🔒 Security Analysis Results:")
                print(analysis.get("analysis", analysis))
                break
    except Exception as e:
        print(f"❌ Analysis failed: {e}")

async def demo_code_enhancement():
    """Demonstrate AI-powered code enhancement"""
    print("\n" + "=" * 60)
    print("✨ AI-Powered Code Enhancement Demo")
    print("=" * 60)
    
    # Basic contract to enhance
    basic_code = """
contract BasicToken {
    state {
        balances: map<address, u64>;
    }
    
    public fn transfer(to: address, amount: u64) {
        balances[msg_sender()] = balances[msg_sender()] - amount;
        balances[to] = balances[to] + amount;
    }
}
"""
    
    config = AIConfig()
    engine = AdvancedAIEngine(config)
    
    print("\n🔧 Enhancing contract with security features...")
    print("-" * 40)
    
    try:
        enhanced = await engine.enhance_code(basic_code, "security")
        print("\n✅ Enhanced Contract:")
        print(enhanced)
    except Exception as e:
        print(f"❌ Enhancement failed: {e}")

async def demo_code_explanation():
    """Demonstrate AI-powered code explanation"""
    print("\n" + "=" * 60)
    print("📚 AI-Powered Code Explanation Demo")
    print("=" * 60)
    
    complex_code = """
contract UniswapV2Pair {
    state {
        reserve0: u112;
        reserve1: u112;
        kLast: u256;
        totalSupply: u256;
    }
    
    public fn swap(amount0Out: u256, amount1Out: u256, to: address) {
        require(amount0Out > 0 || amount1Out > 0, "INSUFFICIENT_OUTPUT");
        
        let balance0 = getBalance(token0);
        let balance1 = getBalance(token1);
        
        let amount0In = balance0 > reserve0 - amount0Out ? 
                        balance0 - (reserve0 - amount0Out) : 0;
        let amount1In = balance1 > reserve1 - amount1Out ? 
                        balance1 - (reserve1 - amount1Out) : 0;
        
        require(amount0In > 0 || amount1In > 0, "INSUFFICIENT_INPUT");
        
        let balance0Adjusted = balance0 * 1000 - amount0In * 3;
        let balance1Adjusted = balance1 * 1000 - amount1In * 3;
        
        require(balance0Adjusted * balance1Adjusted >= 
                reserve0 * reserve1 * 1000000, "K");
        
        update(balance0, balance1);
        emit Swap(msg_sender(), amount0In, amount1In, amount0Out, amount1Out, to);
    }
}
"""
    
    config = AIConfig()
    engine = AdvancedAIEngine(config)
    
    print("\n📖 Explaining complex AMM swap logic...")
    print("-" * 40)
    
    try:
        explanation = await engine.explain_code(complex_code)
        print("\n✅ Explanation:")
        print(explanation)
    except Exception as e:
        print(f"❌ Explanation failed: {e}")

def check_api_keys():
    """Check if API keys are configured"""
    keys_found = []
    
    if os.getenv("OPENAI_API_KEY"):
        keys_found.append("OpenAI")
    if os.getenv("ANTHROPIC_API_KEY"):
        keys_found.append("Anthropic")
    if os.getenv("GOOGLE_API_KEY"):
        keys_found.append("Google")
    
    if not keys_found:
        print("\n⚠️  WARNING: No AI API keys found in environment variables!")
        print("Please set at least one of the following:")
        print("  - OPENAI_API_KEY")
        print("  - ANTHROPIC_API_KEY")
        print("  - GOOGLE_API_KEY")
        print("\nExample:")
        print("  export OPENAI_API_KEY='your-api-key-here'")
        return False
    else:
        print(f"\n✅ Found API keys for: {', '.join(keys_found)}")
        return True

async def main():
    """Main demo function"""
    print("\n" + "🌟" * 30)
    print("   CrossChain DSL - AI Integration Demo")
    print("🌟" * 30)
    
    # Check for API keys
    if not check_api_keys():
        print("\n❌ Demo requires at least one AI provider API key")
        print("Exiting...")
        return
    
    print("\nSelect a demo to run:")
    print("1. AI Code Generation")
    print("2. AI Code Analysis")
    print("3. AI Code Enhancement")
    print("4. AI Code Explanation")
    print("5. Run All Demos")
    
    choice = input("\nEnter your choice (1-5): ").strip()
    
    if choice == "1":
        await demo_ai_code_generation()
    elif choice == "2":
        await demo_code_analysis()
    elif choice == "3":
        await demo_code_enhancement()
    elif choice == "4":
        await demo_code_explanation()
    elif choice == "5":
        await demo_ai_code_generation()
        await demo_code_analysis()
        await demo_code_enhancement()
        await demo_code_explanation()
    else:
        print("Invalid choice. Please run the script again.")
    
    print("\n" + "=" * 60)
    print("✅ Demo completed!")
    print("=" * 60)

if __name__ == "__main__":
    asyncio.run(main())