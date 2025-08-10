# CrossChain DSL Compiler

## 概述

这是一个革命性的编译器，允许您使用**一套统一的 DSL 代码**，自动生成并部署到 Solana、Aptos 和 Sui 区块链。

## 核心理念

不再需要为每个区块链编写不同的代码！通过我们的 CrossChain DSL（`.ccdsl` 文件），您可以：

1. **编写一次**：使用简洁的 DSL 语法定义业务逻辑
2. **编译到多链**：自动生成 Solana (Rust) 和 Move 代码
3. **保持一致性**：确保所有链上的逻辑完全相同

## DSL 语法示例

```dsl
contract Token {
    state {
        total_supply: u64;
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
```

## 编译结果

### Solana (Anchor) 输出
```rust
#[program]
pub mod token {
    pub fn transfer(ctx: Context<Transfer>, amount: u64) -> Result<()> {
        // 自动生成的 Solana 代码
    }
}
```

### Aptos Move 输出
```move
module token_addr::token {
    public entry fun transfer(from: &signer, to: address, amount: u64) {
        // 自动生成的 Move 代码
    }
}
```

## 安装和使用

### 1. 构建编译器
```bash
cd dsl-compiler
cargo build --release
```

### 2. 编写 DSL 文件
```bash
# 生成示例文件
./target/release/ccdsl example -o my_token.ccdsl

# 编辑您的合约
vim my_token.ccdsl
```

### 3. 编译到目标平台
```bash
# 编译到所有平台
./target/release/ccdsl compile -i my_token.ccdsl -t all

# 仅编译到 Solana
./target/release/ccdsl compile -i my_token.ccdsl -t solana

# 仅编译到 Aptos
./target/release/ccdsl compile -i my_token.ccdsl -t aptos
```

### 4. 部署生成的代码
```bash
# Solana 部署
cd output/solana
anchor build && anchor deploy

# Aptos 部署
cd output/aptos
aptos move compile && aptos move publish

# Sui 部署
cd output/sui
sui move build && sui client publish
```

## 技术架构

```
DSL 源文件 (.ccdsl)
        ↓
    词法分析器 (Pest)
        ↓
    语法分析器 → AST
        ↓
    语义分析器
        ↓
  中间表示 (IR)
    ↙     ↓     ↘
Solana   Aptos   Sui
代码生成器
    ↓      ↓      ↓
 Rust   Move   Move
```

## 支持的特性

### 数据类型
- ✅ 基础类型：u8, u64, u128, bool, address, string
- ✅ 集合类型：map, vec
- ✅ 自定义结构体

### 控制流
- ✅ if/else 条件
- ✅ require 断言
- ✅ 函数调用

### 区块链特性
- ✅ 状态存储
- ✅ 事件发射
- ✅ 访问控制
- ✅ 跨链地址兼容

## 类型映射

| DSL 类型 | Solana (Rust) | Aptos/Sui (Move) |
|---------|---------------|------------------|
| u64 | u64 | u64 |
| address | Pubkey | address |
| string | String | vector<u8> |
| map<K,V> | HashMap<K,V> | SimpleMap<K,V> |
| vec<T> | Vec<T> | vector<T> |

## 高级特性

### 1. 平台特定优化
编译器会根据目标平台自动优化：
- Solana：利用账户模型和并行执行
- Aptos：使用资源模型和 Block-STM
- Sui：优化对象存储和并行处理

### 2. 安全检查
- 编译时类型检查
- 资源安全验证
- 跨链一致性验证

### 3. 调试支持
- 源码映射
- 错误追踪
- 性能分析

## 路线图

- [x] Phase 1: 基础 DSL 语法和解析器
- [x] Phase 2: Solana 代码生成器
- [x] Phase 3: Move 代码生成器
- [ ] Phase 4: 完整的类型系统
- [ ] Phase 5: 优化器
- [ ] Phase 6: IDE 支持（VS Code 插件）
- [ ] Phase 7: 形式化验证

## 贡献

欢迎贡献！请查看 [CONTRIBUTING.md](../CONTRIBUTING.md)

## 许可证

MIT License