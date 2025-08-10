# Solana-Move 跨链部署系统

## 架构概述

本项目采用**统一接口 + 双重实现**的架构，而非直接代码转译。这种方法更实际可行，能够：

1. **保持各链特性**：充分利用 Solana 和 Move 各自的优势
2. **统一开发体验**：通过抽象接口提供一致的 API
3. **灵活扩展**：易于添加新的区块链支持

## 为什么不采用直接转译？

### 根本性差异

| 特性 | Solana | Move (Aptos/Sui) |
|------|--------|------------------|
| 编程模型 | 账户模型 | 资源/对象模型 |
| 状态存储 | 程序与数据分离 | 模块内聚 |
| 并发执行 | Sealevel (账户级) | Block-STM/对象级 |
| 类型系统 | Rust 借用检查 | Move 资源语义 |
| 运行时 | BPF/SBF | MoveVM |

这些差异使得直接转译在技术上不可行且不安全。

## 项目结构

```
.
├── core/                    # 核心抽象层
│   ├── traits/             # 统一接口定义
│   ├── types/              # 共享类型系统
│   └── protocols/          # 跨链协议
│
├── solana-impl/            # Solana 实现
│   ├── programs/           # Anchor 程序
│   ├── client/             # JS/TS 客户端
│   └── tests/              # 测试套件
│
├── move-impl/              # Move 实现
│   ├── aptos/              # Aptos 模块
│   ├── sui/                # Sui 模块
│   └── tests/              # Move 测试
│
├── bridge/                 # 跨链桥接
│   ├── wormhole-adapter/   # Wormhole 集成
│   ├── chain-signatures/   # NEAR 链签名
│   └── message-passing/    # 消息协议
│
└── tools/                  # 开发工具
    ├── code-gen/           # 代码生成器
    ├── deployer/           # 部署脚本
    └── validator/          # 验证工具
```

## 核心设计理念

### 1. 统一接口层

```rust
// 所有链实现相同的业务接口
pub trait TokenOperations {
    fn transfer(&self, from: Address, to: Address, amount: u64) -> Result<()>;
    fn mint(&self, to: Address, amount: u64) -> Result<()>;
    // ...
}
```

### 2. 链特定实现

- **Solana**: 使用 Anchor 框架，利用账户模型
- **Aptos**: 使用 Move 资源模型，Block-STM 并行
- **Sui**: 使用对象模型，支持更细粒度并行

### 3. 跨链通信

采用成熟的跨链协议：
- **Wormhole**: 去中心化跨链消息传递
- **NEAR Chain Signatures**: 统一密钥管理
- **Intent-Based**: 用户友好的跨链操作

## 快速开始

### 安装依赖

```bash
# Solana 工具链
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest

# Aptos CLI
curl -fsSL "https://aptos.dev/scripts/install_cli.py" | python3

# Sui CLI
cargo install --locked --git https://github.com/MystenLabs/sui.git --branch mainnet sui
```

### 构建项目

```bash
# 构建 Solana 程序
cd solana-impl/programs
anchor build

# 构建 Aptos 模块
cd move-impl/aptos
aptos move compile

# 构建 Sui 模块
cd move-impl/sui
sui move build
```

## 开发路线图

### Phase 1: 基础框架 (当前)
- [x] 设计统一接口
- [x] 创建项目结构
- [ ] 实现基础代币功能

### Phase 2: 跨链集成
- [ ] Wormhole 集成
- [ ] NEAR Chain Signatures
- [ ] 跨链状态同步

### Phase 3: 生产就绪
- [ ] 安全审计
- [ ] 性能优化
- [ ] 开发者工具

## 技术优势

1. **真正的多链部署**：一套业务逻辑，多链运行
2. **保持原生性能**：不牺牲各链的性能优势
3. **安全可靠**：利用成熟的跨链基础设施
4. **开发友好**：统一的 API 和工具链

## 贡献指南

欢迎贡献代码！请查看 [CONTRIBUTING.md](./CONTRIBUTING.md) 了解详情。

## 许可证

MIT License