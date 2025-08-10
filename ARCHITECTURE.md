# CrossChain DSL 架构文档

## 🎯 核心目标

实现"一套代码，多链部署"的愿景，通过统一的 DSL 语言编译到 Solana、Aptos、Sui 等多个区块链平台。

## 🏗️ 系统架构

```
┌─────────────────────────────────────────────────┐
│              CrossChain DSL (.ccdsl)            │
│         统一的智能合约定义语言                    │
└────────────────────┬────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────┐
│                DSL Compiler                      │
│  ┌──────────────────────────────────────────┐   │
│  │  1. Lexer (词法分析) - Pest Parser       │   │
│  ├──────────────────────────────────────────┤   │
│  │  2. Parser (语法分析) → AST              │   │
│  ├──────────────────────────────────────────┤   │
│  │  3. Semantic Analysis (语义分析)         │   │
│  ├──────────────────────────────────────────┤   │
│  │  4. IR Generation (中间表示生成)         │   │
│  └──────────────────────────────────────────┘   │
└────────────────────┬────────────────────────────┘
                     │
        ┌────────────┼────────────┐
        ▼            ▼            ▼
┌───────────┐ ┌───────────┐ ┌───────────┐
│  Solana   │ │   Aptos   │ │    Sui    │
│ Generator │ │ Generator │ │ Generator │
└─────┬─────┘ └─────┬─────┘ └─────┬─────┘
      │             │             │
      ▼             ▼             ▼
   Rust/       Move Code     Move Code
   Anchor      (Aptos)       (Sui)
```

## 📝 DSL 语言设计

### 核心原则

1. **简洁性**：语法接近主流编程语言，易于学习
2. **表达力**：能够表达所有常见的智能合约逻辑
3. **安全性**：编译时检查，防止常见错误
4. **可移植性**：抽象底层差异，保证跨链一致性

### 语法特性

```dsl
contract TokenContract {
    // 状态变量
    state {
        total_supply: u64;
        balances: map<address, u64>;
        owner: address;
    }
    
    // 公开函数
    public fn transfer(to: address, amount: u64) {
        let from = msg_sender();
        require(balances[from] >= amount);
        
        balances[from] -= amount;
        balances[to] += amount;
        
        emit Transfer(from, to, amount);
    }
    
    // 私有函数
    private fn check_owner() {
        require(msg_sender() == owner);
    }
}
```

## 🔄 编译流程

### 1. 词法分析 (Lexer)
- 使用 Pest parser 框架
- 将源代码分解为 tokens
- 处理注释、空白符等

### 2. 语法分析 (Parser)
- 构建抽象语法树 (AST)
- 语法验证
- 错误报告和恢复

### 3. 语义分析
- 类型检查
- 变量作用域分析
- 函数签名验证
- 资源安全检查

### 4. 中间表示 (IR)
- 平台无关的代码表示
- 优化机会识别
- 便于多目标代码生成

### 5. 代码生成
- **Solana**: 生成 Anchor 框架代码
- **Aptos**: 生成 Move 模块
- **Sui**: 生成 Sui Move 对象

## 🔧 类型系统映射

| DSL 类型 | Solana | Aptos | Sui | 说明 |
|---------|--------|-------|-----|------|
| u8/u64/u128 | u8/u64/u128 | u8/u64/u128 | u8/u64/u128 | 数值类型直接映射 |
| bool | bool | bool | bool | 布尔类型 |
| address | Pubkey | address | address | 账户地址 |
| string | String | vector<u8> | vector<u8> | 字符串 |
| map<K,V> | HashMap<K,V> | SimpleMap<K,V> | Table<K,V> | 键值对映射 |
| vec<T> | Vec<T> | vector<T> | vector<T> | 动态数组 |

## 🎯 关键挑战与解决方案

### 挑战 1: 账户模型差异
- **Solana**: 账户存储数据，程序无状态
- **Move**: 资源/对象模型

**解决方案**: 统一抽象为"状态"概念，编译器自动处理底层差异

### 挑战 2: 并发模型不同
- **Solana**: Sealevel 并行执行
- **Aptos**: Block-STM
- **Sui**: 对象级并行

**解决方案**: DSL 层面不暴露并发细节，由编译器优化

### 挑战 3: 系统调用差异
- 不同链有不同的内置函数

**解决方案**: 提供统一的标准库抽象

## 📊 项目结构

```
solana-move/
├── dsl-compiler/           # DSL 编译器核心
│   ├── src/
│   │   ├── lib.rs         # 解析器和 AST
│   │   ├── main.rs        # CLI 入口
│   │   └── codegen/       # 代码生成器
│   │       ├── solana.rs  # Solana 代码生成
│   │       └── move_gen.rs # Move 代码生成
│   ├── grammar.pest       # DSL 语法定义
│   └── examples/          # DSL 示例
│       └── token.ccdsl    # 代币合约示例
│
├── core/                  # 共享抽象（备用方案）
│   ├── traits/           # 统一接口
│   └── types/            # 共享类型
│
├── solana-impl/          # Solana 实现示例
├── move-impl/            # Move 实现示例
│   ├── aptos/
│   └── sui/
│
├── tools/                # 辅助工具
│   └── deployer/         # 部署脚本
│
└── demo.sh              # 演示脚本
```

## 🚀 使用流程

1. **编写 DSL 代码**
```bash
vim my_contract.ccdsl
```

2. **编译到目标平台**
```bash
ccdsl compile -i my_contract.ccdsl -t all
```

3. **部署生成的代码**
```bash
# Solana
cd output/solana && anchor deploy

# Aptos
cd output/aptos && aptos move publish

# Sui
cd output/sui && sui client publish
```

## 📈 发展路线图

### Phase 1: MVP (已完成) ✅
- [x] DSL 语法设计
- [x] 基础解析器
- [x] Solana 代码生成
- [x] Move 代码生成

### Phase 2: 完整实现
- [ ] 完整的类型系统
- [ ] 高级语言特性（循环、复杂数据结构）
- [ ] 优化器
- [ ] 错误处理改进

### Phase 3: 生产就绪
- [ ] 形式化验证
- [ ] 安全审计工具
- [ ] IDE 插件（VS Code、IntelliJ）
- [ ] 调试器支持

### Phase 4: 生态系统
- [ ] 标准库扩展
- [ ] 包管理器
- [ ] 测试框架
- [ ] 文档生成器

## 🔐 安全考虑

1. **编译时验证**
   - 类型安全
   - 资源安全（防止资源泄露）
   - 整数溢出检查

2. **跨链一致性**
   - 确保相同输入产生相同逻辑
   - 处理链特定的限制

3. **审计友好**
   - 生成的代码可读
   - 保留源码映射
   - 支持形式化验证

## 💡 创新点

1. **真正的多链部署**：不是简单的包装器，而是真正的代码生成
2. **保持原生性能**：生成的代码与手写代码性能相当
3. **渐进式采用**：可以与现有代码混合使用
4. **开发者友好**：熟悉的语法，强大的工具链

## 🤝 贡献指南

我们欢迎社区贡献！重点领域：
- DSL 语言改进
- 新平台支持（如 Cosmos、Near）
- 优化器改进
- 工具链增强

## 📚 参考资料

- [Solana 文档](https://docs.solana.com)
- [Move 语言规范](https://github.com/move-language/move)
- [Pest Parser](https://pest.rs)
- [编译原理](https://craftinginterpreters.com)

---

**这是区块链开发的未来：一次编写，处处运行！**