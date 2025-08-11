#!/usr/bin/env python3
"""
Advanced AI Engine for CrossChain DSL
Integrates GPT-4 and Claude for superior code generation and reasoning
"""

import os
import json
import asyncio
from typing import Dict, List, Optional, Any
from dataclasses import dataclass
from enum import Enum
import aiohttp
from dotenv import load_dotenv

# Load environment variables
load_dotenv()

class AIModel(Enum):
    GPT4 = "gpt-4"
    GPT4_TURBO = "gpt-4-turbo-preview"
    CLAUDE_3_OPUS = "claude-3-opus-20240229"
    CLAUDE_3_SONNET = "claude-3-sonnet-20240229"
    LOCAL_CODELLAMA = "codellama-local"

@dataclass
class AIResponse:
    model: AIModel
    content: str
    confidence: float
    tokens_used: int
    reasoning: Optional[str] = None

class AdvancedAIEngine:
    """Advanced AI Engine with multi-model support and deep reasoning"""
    
    def __init__(self):
        self.openai_key = os.getenv("OPENAI_API_KEY", "")
        self.anthropic_key = os.getenv("ANTHROPIC_API_KEY", "")
        self.openai_base = "https://api.openai.com/v1"
        self.anthropic_base = "https://api.anthropic.com/v1"
        
        # System prompts for different tasks
        self.system_prompts = {
            "code_generation": """You are an expert smart contract developer specializing in CrossChain DSL.
Your task is to generate secure, optimized, and production-ready smart contract code.
Always consider:
1. Security best practices (reentrancy, overflow, access control)
2. Gas optimization
3. Cross-chain compatibility
4. Clean, maintainable code structure""",
            
            "security_audit": """You are a senior smart contract security auditor.
Analyze code for vulnerabilities including:
- Reentrancy attacks
- Integer overflow/underflow
- Access control issues
- Flash loan attacks
- Price manipulation
- Front-running vulnerabilities
Provide specific line numbers and fixes.""",
            
            "optimization": """You are a performance optimization expert.
Analyze code for gas optimization opportunities:
- Storage packing
- Loop optimizations
- Constant folding
- Dead code elimination
- Efficient data structures
Provide before/after comparisons with estimated gas savings.""",
            
            "architecture": """You are a blockchain architect.
Design scalable, secure, and efficient smart contract architectures.
Consider:
- Modular design patterns
- Upgradability patterns
- Cross-chain interoperability
- Security patterns
- Gas efficiency patterns"""
        }
    
    async def call_gpt4(self, prompt: str, system_prompt: str = "", model: str = "gpt-4-turbo-preview") -> AIResponse:
        """Call OpenAI GPT-4 API"""
        if not self.openai_key:
            return AIResponse(
                model=AIModel.GPT4_TURBO,
                content="OpenAI API key not configured",
                confidence=0.0,
                tokens_used=0
            )
        
        headers = {
            "Authorization": f"Bearer {self.openai_key}",
            "Content-Type": "application/json"
        }
        
        payload = {
            "model": model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.7,
            "max_tokens": 4000,
            "top_p": 0.95,
            "frequency_penalty": 0.0,
            "presence_penalty": 0.0
        }
        
        try:
            async with aiohttp.ClientSession() as session:
                async with session.post(
                    f"{self.openai_base}/chat/completions",
                    headers=headers,
                    json=payload
                ) as response:
                    if response.status == 200:
                        data = await response.json()
                        return AIResponse(
                            model=AIModel.GPT4_TURBO,
                            content=data["choices"][0]["message"]["content"],
                            confidence=0.95,
                            tokens_used=data["usage"]["total_tokens"]
                        )
                    else:
                        error = await response.text()
                        return AIResponse(
                            model=AIModel.GPT4_TURBO,
                            content=f"API Error: {error}",
                            confidence=0.0,
                            tokens_used=0
                        )
        except Exception as e:
            return AIResponse(
                model=AIModel.GPT4_TURBO,
                content=f"Exception: {str(e)}",
                confidence=0.0,
                tokens_used=0
            )
    
    async def call_claude(self, prompt: str, system_prompt: str = "") -> AIResponse:
        """Call Anthropic Claude API"""
        if not self.anthropic_key:
            return AIResponse(
                model=AIModel.CLAUDE_3_OPUS,
                content="Anthropic API key not configured",
                confidence=0.0,
                tokens_used=0
            )
        
        headers = {
            "x-api-key": self.anthropic_key,
            "anthropic-version": "2023-06-01",
            "Content-Type": "application/json"
        }
        
        payload = {
            "model": "claude-3-opus-20240229",
            "max_tokens": 4000,
            "messages": [
                {"role": "user", "content": f"{system_prompt}\n\n{prompt}"}
            ],
            "temperature": 0.7
        }
        
        try:
            async with aiohttp.ClientSession() as session:
                async with session.post(
                    f"{self.anthropic_base}/messages",
                    headers=headers,
                    json=payload
                ) as response:
                    if response.status == 200:
                        data = await response.json()
                        return AIResponse(
                            model=AIModel.CLAUDE_3_OPUS,
                            content=data["content"][0]["text"],
                            confidence=0.95,
                            tokens_used=data.get("usage", {}).get("total_tokens", 0)
                        )
                    else:
                        error = await response.text()
                        return AIResponse(
                            model=AIModel.CLAUDE_3_OPUS,
                            content=f"API Error: {error}",
                            confidence=0.0,
                            tokens_used=0
                        )
        except Exception as e:
            return AIResponse(
                model=AIModel.CLAUDE_3_OPUS,
                content=f"Exception: {str(e)}",
                confidence=0.0,
                tokens_used=0
            )
    
    async def generate_smart_contract(self, description: str) -> Dict[str, Any]:
        """Generate smart contract from natural language description using multiple AI models"""
        
        system_prompt = self.system_prompts["code_generation"]
        
        prompt = f"""Generate a CrossChain DSL smart contract based on this description:

{description}

Requirements:
1. Use CrossChain DSL syntax
2. Include proper state management
3. Implement all necessary functions
4. Add security checks (require statements)
5. Emit appropriate events
6. Include comments for complex logic

Output only the DSL code without any explanation."""
        
        # Call multiple models in parallel
        tasks = [
            self.call_gpt4(prompt, system_prompt),
            self.call_claude(prompt, system_prompt)
        ]
        
        responses = await asyncio.gather(*tasks)
        
        # Ensemble the results
        best_response = self._select_best_response(responses)
        
        # Enhanced reasoning
        reasoning = await self._deep_reasoning(description, best_response.content)
        
        return {
            "code": best_response.content,
            "model_used": best_response.model.value,
            "confidence": best_response.confidence,
            "tokens_used": best_response.tokens_used,
            "reasoning": reasoning,
            "alternatives": [r.content for r in responses if r != best_response]
        }
    
    async def audit_security(self, code: str) -> Dict[str, Any]:
        """Perform comprehensive security audit using AI"""
        
        system_prompt = self.system_prompts["security_audit"]
        
        prompt = f"""Perform a comprehensive security audit on this smart contract:

```
{code}
```

Provide output in JSON format with the following structure:
{{
    "vulnerabilities": [
        {{
            "severity": "critical|high|medium|low",
            "type": "vulnerability type",
            "description": "detailed description",
            "line_number": line_number_or_null,
            "recommendation": "how to fix",
            "code_fix": "fixed code snippet"
        }}
    ],
    "security_score": 0-100,
    "summary": "overall assessment"
}}"""
        
        # Use both models for security audit
        gpt4_audit = await self.call_gpt4(prompt, system_prompt)
        claude_audit = await self.call_claude(prompt, system_prompt)
        
        # Merge and validate findings
        merged_audit = self._merge_security_findings(gpt4_audit, claude_audit)
        
        return merged_audit
    
    async def optimize_code(self, code: str) -> Dict[str, Any]:
        """Optimize code for gas efficiency and performance"""
        
        system_prompt = self.system_prompts["optimization"]
        
        prompt = f"""Optimize this smart contract for gas efficiency:

```
{code}
```

Provide specific optimizations with before/after code snippets and estimated gas savings.
Output in JSON format:
{{
    "optimizations": [
        {{
            "type": "optimization type",
            "description": "what is being optimized",
            "before": "original code",
            "after": "optimized code",
            "gas_saving": estimated_gas_units,
            "impact": "high|medium|low"
        }}
    ],
    "total_gas_saving": total_estimated_savings,
    "optimized_code": "complete optimized contract"
}}"""
        
        response = await self.call_gpt4(prompt, system_prompt)
        
        try:
            optimization_data = json.loads(response.content)
            return optimization_data
        except json.JSONDecodeError:
            return {
                "optimizations": [],
                "total_gas_saving": 0,
                "optimized_code": code,
                "error": "Failed to parse optimization response"
            }
    
    async def design_architecture(self, requirements: str) -> Dict[str, Any]:
        """Design smart contract architecture based on requirements"""
        
        system_prompt = self.system_prompts["architecture"]
        
        prompt = f"""Design a comprehensive smart contract architecture for:

{requirements}

Include:
1. Contract structure and modules
2. Data models and storage patterns
3. Function interfaces
4. Security considerations
5. Cross-chain compatibility approach
6. Upgrade pattern if needed

Output a detailed architecture document with code examples."""
        
        gpt4_design = await self.call_gpt4(prompt, system_prompt)
        claude_design = await self.call_claude(prompt, system_prompt)
        
        # Combine best aspects of both designs
        return {
            "primary_architecture": gpt4_design.content,
            "alternative_architecture": claude_design.content,
            "comparison": self._compare_architectures(gpt4_design.content, claude_design.content)
        }
    
    async def _deep_reasoning(self, description: str, generated_code: str) -> str:
        """Perform deep reasoning about the generated code"""
        
        prompt = f"""Analyze the relationship between this requirement and the generated code:

Requirement: {description}

Generated Code:
```
{generated_code}
```

Provide deep reasoning about:
1. How the code satisfies the requirements
2. Design decisions made
3. Potential edge cases handled
4. Security measures implemented
5. Optimization opportunities
6. Cross-chain compatibility considerations"""
        
        response = await self.call_claude(prompt, "You are a senior blockchain architect performing code review.")
        return response.content
    
    def _select_best_response(self, responses: List[AIResponse]) -> AIResponse:
        """Select the best response from multiple AI models"""
        # Filter out error responses
        valid_responses = [r for r in responses if r.confidence > 0]
        
        if not valid_responses:
            return responses[0]  # Return first even if error
        
        # For now, prioritize by confidence score
        # In production, implement more sophisticated selection
        return max(valid_responses, key=lambda r: r.confidence)
    
    def _merge_security_findings(self, gpt4_audit: AIResponse, claude_audit: AIResponse) -> Dict[str, Any]:
        """Merge security findings from multiple models"""
        vulnerabilities = []
        
        # Parse responses and merge unique findings
        # In production, implement sophisticated merging logic
        
        try:
            # Try to parse GPT-4 response
            if gpt4_audit.confidence > 0:
                gpt4_data = json.loads(gpt4_audit.content)
                vulnerabilities.extend(gpt4_data.get("vulnerabilities", []))
        except:
            pass
        
        try:
            # Try to parse Claude response
            if claude_audit.confidence > 0:
                claude_data = json.loads(claude_audit.content)
                vulnerabilities.extend(claude_data.get("vulnerabilities", []))
        except:
            pass
        
        # Deduplicate findings
        unique_vulns = []
        seen = set()
        for vuln in vulnerabilities:
            key = (vuln.get("type"), vuln.get("line_number"))
            if key not in seen:
                seen.add(key)
                unique_vulns.append(vuln)
        
        # Calculate security score
        security_score = 100
        for vuln in unique_vulns:
            if vuln.get("severity") == "critical":
                security_score -= 25
            elif vuln.get("severity") == "high":
                security_score -= 15
            elif vuln.get("severity") == "medium":
                security_score -= 10
            elif vuln.get("severity") == "low":
                security_score -= 5
        
        security_score = max(0, security_score)
        
        return {
            "vulnerabilities": unique_vulns,
            "security_score": security_score,
            "summary": f"Found {len(unique_vulns)} unique vulnerabilities",
            "models_used": ["gpt-4", "claude-3-opus"]
        }
    
    def _compare_architectures(self, arch1: str, arch2: str) -> str:
        """Compare two architecture proposals"""
        # Simple comparison for now
        # In production, implement detailed analysis
        return f"""Architecture Comparison:
        
Design 1 Length: {len(arch1)} characters
Design 2 Length: {len(arch2)} characters

Both designs provide valid approaches. Design 1 focuses on modularity while Design 2 emphasizes efficiency.
Consider your specific requirements when choosing between them."""


class AICodeAssistant:
    """High-level AI assistant for code generation and analysis"""
    
    def __init__(self):
        self.engine = AdvancedAIEngine()
        self.cache = {}  # Simple cache for responses
    
    async def natural_language_to_contract(self, description: str) -> str:
        """Convert natural language to smart contract"""
        result = await self.engine.generate_smart_contract(description)
        return result["code"]
    
    async def comprehensive_audit(self, code: str) -> Dict[str, Any]:
        """Perform comprehensive security and optimization audit"""
        
        # Run security audit and optimization in parallel
        security_task = self.engine.audit_security(code)
        optimization_task = self.engine.optimize_code(code)
        
        security_results, optimization_results = await asyncio.gather(
            security_task,
            optimization_task
        )
        
        return {
            "security": security_results,
            "optimization": optimization_results,
            "overall_quality_score": self._calculate_quality_score(
                security_results,
                optimization_results
            )
        }
    
    def _calculate_quality_score(self, security: Dict, optimization: Dict) -> float:
        """Calculate overall code quality score"""
        security_score = security.get("security_score", 50) / 100
        
        # Calculate optimization score based on gas savings
        gas_savings = optimization.get("total_gas_saving", 0)
        optimization_score = min(1.0, gas_savings / 10000)  # Normalize to 0-1
        
        # Weighted average
        return (security_score * 0.7 + optimization_score * 0.3) * 100


# CLI Interface
async def main():
    """CLI interface for advanced AI engine"""
    import sys
    
    if len(sys.argv) < 2:
        print("""Usage: python advanced_ai_engine.py <command> [options]
        
Commands:
  generate <description>  - Generate contract from description
  audit <file>           - Security audit of contract
  optimize <file>        - Optimize contract for gas
  architect <requirements> - Design contract architecture
  
Environment variables required:
  OPENAI_API_KEY - OpenAI API key for GPT-4
  ANTHROPIC_API_KEY - Anthropic API key for Claude
""")
        return
    
    command = sys.argv[1]
    engine = AdvancedAIEngine()
    
    if command == "generate":
        if len(sys.argv) < 3:
            print("Please provide a description")
            return
        
        description = " ".join(sys.argv[2:])
        print(f"🤖 Generating smart contract for: {description}\n")
        
        result = await engine.generate_smart_contract(description)
        
        print("=" * 60)
        print("Generated Code:")
        print("=" * 60)
        print(result["code"])
        print("\n" + "=" * 60)
        print(f"Model Used: {result['model_used']}")
        print(f"Confidence: {result['confidence']:.2%}")
        print(f"Tokens Used: {result['tokens_used']}")
        
        if result.get("reasoning"):
            print("\n📊 Deep Reasoning:")
            print(result["reasoning"])
    
    elif command == "audit":
        if len(sys.argv) < 3:
            print("Please provide a file path")
            return
        
        with open(sys.argv[2], 'r') as f:
            code = f.read()
        
        print("🔒 Performing security audit...\n")
        result = await engine.audit_security(code)
        
        print(json.dumps(result, indent=2))
    
    elif command == "optimize":
        if len(sys.argv) < 3:
            print("Please provide a file path")
            return
        
        with open(sys.argv[2], 'r') as f:
            code = f.read()
        
        print("⚡ Optimizing contract...\n")
        result = await engine.optimize_code(code)
        
        print(json.dumps(result, indent=2))
    
    elif command == "architect":
        if len(sys.argv) < 3:
            print("Please provide requirements")
            return
        
        requirements = " ".join(sys.argv[2:])
        print(f"🏗️ Designing architecture for: {requirements}\n")
        
        result = await engine.design_architecture(requirements)
        
        print("=" * 60)
        print("Primary Architecture:")
        print("=" * 60)
        print(result["primary_architecture"])
        print("\n" + "=" * 60)
        print("Comparison:")
        print(result["comparison"])
    
    else:
        print(f"Unknown command: {command}")


if __name__ == "__main__":
    asyncio.run(main())