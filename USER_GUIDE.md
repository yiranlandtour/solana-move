# 📚 CrossChain DSL 用户使用指南

## 🚀 快速开始

### 系统要求
- Python 3.8+
- Rust 1.70+ (可选，用于编译器)
- 2GB+ RAM
- Linux/macOS/Windows

### 安装步骤

1. **克隆项目**
```bash
git clone <repository>
cd solana-move
```

2. **安装 Python 依赖**
```bash
pip install -r ai-integration/requirements.txt
```

3. **配置 AI API 密钥（可选）**
```bash
export OPENAI_API_KEY="your-key-here"
export ANTHROPIC_API_KEY="your-key-here"
```

## 🎮 使用方法

### 1. AI 代码生成

#### 交互式生成
```bash
python3 demo_ai_simple.py
```

选择选项：
- `1` - AI 代码生成
- `2` - AI 安全审计
- `3` - AI 优化建议
- `4` - 运行所有功能
- `5` - 退出

#### 编程式使用
```python
import sys
sys.path.append('ai-integration')
from ai_assistant import AICodeGenerator

async def generate_contract():
    generator = AICodeGenerator()
    code = await generator.generate_from_description(
        "Create a token contract with mint and burn functions"
    )
    print(code)

# 运行
import asyncio
asyncio.run(generate_contract())
```

### 2. 编写 DSL 代码

#### 基本语法

**合约声明**
```dsl
contract MyToken {
    // 合约内容
}
```

**状态变量**
```dsl
state {
    total_supply: u64;
    balances: map<address, u64>;
    owner: address;
}
```

**函数定义**
```dsl
public fn transfer(to: address, amount: u64) {
    require(balances[msg_sender()] >= amount, "Insufficient balance");
    balances[msg_sender()] = balances[msg_sender()] - amount;
    balances[to] = balances[to] + amount;
    emit Transfer(msg_sender(), to, amount);
}
```

**结构体定义**
```dsl
struct Pool {
    token_a: address;
    token_b: address;
    reserve_a: u128;
    reserve_b: u128;
}
```

### 3. 完整示例

#### 代币合约
```dsl
contract ERC20Token {
    state {
        name: string;
        symbol: string;
        decimals: u8;
        total_supply: u256;
        balances: map<address, u256>;
        allowances: map<address, map<address, u256>>;
    }
    
    public fn initialize(
        _name: string,
        _symbol: string,
        _decimals: u8,
        _initial_supply: u256
    ) {
        name = _name;
        symbol = _symbol;
        decimals = _decimals;
        total_supply = _initial_supply;
        balances[msg_sender()] = _initial_supply;
        
        emit Transfer(address(0), msg_sender(), _initial_supply);
    }
    
    public fn transfer(to: address, amount: u256) -> bool {
        require(to != address(0), "Invalid recipient");
        require(balances[msg_sender()] >= amount, "Insufficient balance");
        
        balances[msg_sender()] = balances[msg_sender()] - amount;
        balances[to] = balances[to] + amount;
        
        emit Transfer(msg_sender(), to, amount);
        return true;
    }
    
    public fn approve(spender: address, amount: u256) -> bool {
        require(spender != address(0), "Invalid spender");
        
        allowances[msg_sender()][spender] = amount;
        emit Approval(msg_sender(), spender, amount);
        return true;
    }
    
    public fn transfer_from(
        from: address,
        to: address,
        amount: u256
    ) -> bool {
        require(allowances[from][msg_sender()] >= amount, "Allowance exceeded");
        require(balances[from] >= amount, "Insufficient balance");
        
        balances[from] = balances[from] - amount;
        balances[to] = balances[to] + amount;
        allowances[from][msg_sender()] = allowances[from][msg_sender()] - amount;
        
        emit Transfer(from, to, amount);
        return true;
    }
    
    public fn balance_of(account: address) -> u256 {
        return balances[account];
    }
}
```

#### AMM DEX 合约
```dsl
contract AMMSwap {
    state {
        pools: map<address, Pool>;
        liquidity: map<address, map<address, u128>>;
        fee_rate: u16;  // 0.3% = 30
    }
    
    struct Pool {
        token0: address;
        token1: address;
        reserve0: u128;
        reserve1: u128;
        k_last: u256;
    }
    
    public fn swap(
        token_in: address,
        token_out: address,
        amount_in: u128
    ) -> u128 {
        let pool = pools[pair_id(token_in, token_out)];
        require(pool.reserve0 > 0 && pool.reserve1 > 0, "Pool not initialized");
        
        let (reserve_in, reserve_out) = get_reserves(pool, token_in);
        
        // Calculate output amount with fee
        let amount_in_with_fee = amount_in * (10000 - fee_rate);
        let numerator = amount_in_with_fee * reserve_out;
        let denominator = reserve_in * 10000 + amount_in_with_fee;
        let amount_out = numerator / denominator;
        
        require(amount_out > 0, "Insufficient output amount");
        
        // Update reserves
        update_reserves(pool, token_in, reserve_in + amount_in, reserve_out - amount_out);
        
        emit Swap(msg_sender(), amount_in, amount_out, token_in, token_out);
        return amount_out;
    }
    
    public fn add_liquidity(
        token0: address,
        token1: address,
        amount0: u128,
        amount1: u128
    ) -> u128 {
        let pool = pools[pair_id(token0, token1)];
        
        let liquidity_minted;
        if pool.reserve0 == 0 && pool.reserve1 == 0 {
            // First liquidity provider
            liquidity_minted = sqrt(amount0 * amount1);
        } else {
            // Subsequent providers
            let liquidity0 = amount0 * total_liquidity / pool.reserve0;
            let liquidity1 = amount1 * total_liquidity / pool.reserve1;
            liquidity_minted = min(liquidity0, liquidity1);
        }
        
        require(liquidity_minted > 0, "Insufficient liquidity minted");
        
        // Update state
        pool.reserve0 = pool.reserve0 + amount0;
        pool.reserve1 = pool.reserve1 + amount1;
        liquidity[pair_id(token0, token1)][msg_sender()] += liquidity_minted;
        
        emit LiquidityAdded(msg_sender(), amount0, amount1, liquidity_minted);
        return liquidity_minted;
    }
}
```

### 4. 安全审计

运行安全审计：
```python
from ai_assistant import AISecurityAuditor

async def audit_contract(code):
    auditor = AISecurityAuditor()
    issues = await auditor.audit_contract(code)
    
    for issue in issues:
        print(f"[{issue.level.value}] {issue.category}")
        print(f"  {issue.description}")
        if issue.suggestion:
            print(f"  Suggestion: {issue.suggestion}")
```

### 5. 代码优化

获取优化建议：
```python
from ai_assistant import AIOptimizer

optimizer = AIOptimizer()
suggestions = optimizer.analyze_gas_usage(contract_code)

for suggestion in suggestions:
    print(f"{suggestion.category}: {suggestion.description}")
    if suggestion.gas_saving:
        print(f"  Estimated savings: {suggestion.gas_saving} gas")
```

## 🛠️ 高级功能

### 自定义 AI 提示
```python
generator = AICodeGenerator()
generator.custom_prompt = """
Create a contract with the following requirements:
- Must be upgradeable
- Include pause mechanism
- Multi-signature support
"""
code = await generator.generate_from_description("governance contract")
```

### 批量处理
```python
contracts = ["token", "nft", "dao", "defi"]
results = {}

for contract_type in contracts:
    code = await generator.generate_from_description(f"{contract_type} contract")
    results[contract_type] = code
```

### 导出到不同平台
```python
# 生成 Solana 代码
solana_code = generator.generate_for_platform(code, "solana")

# 生成 Aptos 代码
aptos_code = generator.generate_for_platform(code, "aptos")

# 生成 Sui 代码
sui_code = generator.generate_for_platform(code, "sui")
```

## 📊 支持的数据类型

| DSL 类型 | 说明 | 示例 |
|---------|------|------|
| u8, u16, u32, u64, u128, u256 | 无符号整数 | `balance: u64` |
| i8, i16, i32, i64, i128 | 有符号整数 | `profit: i64` |
| bool | 布尔值 | `is_active: bool` |
| address | 地址类型 | `owner: address` |
| string | 字符串 | `name: string` |
| bytes | 字节数组 | `data: bytes` |
| map<K, V> | 映射 | `balances: map<address, u64>` |
| vec<T> | 动态数组 | `items: vec<u64>` |
| array<T, N> | 固定数组 | `data: array<u8, 32>` |
| Option<T> | 可选类型 | `value: Option<u64>` |
| Result<T, E> | 结果类型 | `result: Result<u64, string>` |

## 🔧 故障排除

### 常见问题

**Q: AI 功能不工作？**
A: 确保已设置 API 密钥：
```bash
export OPENAI_API_KEY="your-key"
```

**Q: 导入模块失败？**
A: 添加正确的路径：
```python
import sys
sys.path.append('ai-integration')
```

**Q: 生成的代码为空？**
A: 检查网络连接和 API 配额

### 调试模式
```python
import logging
logging.basicConfig(level=logging.DEBUG)

# 现在会显示详细的调试信息
generator = AICodeGenerator()
```

## 📚 更多资源

- [技术路线图](TECHNICAL_ROADMAP.md)
- [AI 功能详解](AI_FEATURES.md)
- [开发总结](DEVELOPMENT_SUMMARY.md)
- [测试结果](TEST_RESULTS.md)

## 💡 最佳实践

1. **使用类型注解**: 明确指定所有变量类型
2. **添加 require 检查**: 验证所有输入和状态
3. **触发事件**: 记录所有重要的状态变化
4. **模块化设计**: 使用结构体组织相关数据
5. **安全第一**: 始终运行安全审计

## 🤝 贡献

欢迎贡献代码、报告问题或提出建议！

## 📄 许可证

MIT License

---

**文档版本**: v1.0.0  
**更新日期**: 2025-08-21  
**作者**: CrossChain DSL Team