#!/bin/bash

# 跨链部署脚本

set -e

echo "🚀 Cross-Chain Token Deployment Script"
echo "======================================"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 部署到 Solana
deploy_solana() {
    echo -e "${YELLOW}Deploying to Solana...${NC}"
    cd solana-impl/programs
    
    # 构建程序
    anchor build
    
    # 部署到指定网络
    if [ "$1" == "mainnet" ]; then
        anchor deploy --provider.cluster mainnet
    elif [ "$1" == "devnet" ]; then
        anchor deploy --provider.cluster devnet
    else
        anchor deploy --provider.cluster localnet
    fi
    
    echo -e "${GREEN}✓ Solana deployment complete${NC}"
    cd ../..
}

# 部署到 Aptos
deploy_aptos() {
    echo -e "${YELLOW}Deploying to Aptos...${NC}"
    cd move-impl/aptos
    
    # 编译 Move 模块
    aptos move compile
    
    # 部署到指定网络
    if [ "$1" == "mainnet" ]; then
        aptos move publish --profile mainnet
    elif [ "$1" == "testnet" ]; then
        aptos move publish --profile testnet
    else
        aptos move publish --profile local
    fi
    
    echo -e "${GREEN}✓ Aptos deployment complete${NC}"
    cd ../..
}

# 部署到 Sui
deploy_sui() {
    echo -e "${YELLOW}Deploying to Sui...${NC}"
    cd move-impl/sui
    
    # 构建 Sui 包
    sui move build
    
    # 部署到指定网络
    if [ "$1" == "mainnet" ]; then
        sui client publish --gas-budget 100000000
    elif [ "$1" == "testnet" ]; then
        sui client publish --gas-budget 100000000 --env testnet
    else
        sui client publish --gas-budget 100000000 --env localnet
    fi
    
    echo -e "${GREEN}✓ Sui deployment complete${NC}"
    cd ../..
}

# 配置跨链桥
setup_bridge() {
    echo -e "${YELLOW}Setting up cross-chain bridge...${NC}"
    
    # 这里添加 Wormhole 或其他跨链桥的配置
    # 例如：注册合约地址、设置信任关系等
    
    echo -e "${GREEN}✓ Bridge setup complete${NC}"
}

# 主函数
main() {
    NETWORK=${1:-localnet}
    CHAINS=${2:-all}
    
    echo "Network: $NETWORK"
    echo "Chains: $CHAINS"
    echo ""
    
    if [ "$CHAINS" == "all" ] || [ "$CHAINS" == "solana" ]; then
        deploy_solana $NETWORK
    fi
    
    if [ "$CHAINS" == "all" ] || [ "$CHAINS" == "aptos" ]; then
        deploy_aptos $NETWORK
    fi
    
    if [ "$CHAINS" == "all" ] || [ "$CHAINS" == "sui" ]; then
        deploy_sui $NETWORK
    fi
    
    if [ "$CHAINS" == "all" ]; then
        setup_bridge
    fi
    
    echo ""
    echo -e "${GREEN}🎉 Deployment complete!${NC}"
}

# 显示帮助
if [ "$1" == "--help" ] || [ "$1" == "-h" ]; then
    echo "Usage: ./deploy.sh [network] [chains]"
    echo ""
    echo "Networks:"
    echo "  localnet  - Deploy to local network (default)"
    echo "  testnet   - Deploy to testnet"
    echo "  mainnet   - Deploy to mainnet"
    echo ""
    echo "Chains:"
    echo "  all       - Deploy to all chains (default)"
    echo "  solana    - Deploy only to Solana"
    echo "  aptos     - Deploy only to Aptos"
    echo "  sui       - Deploy only to Sui"
    echo ""
    echo "Examples:"
    echo "  ./deploy.sh                    # Deploy to all chains on localnet"
    echo "  ./deploy.sh testnet solana     # Deploy to Solana testnet only"
    echo "  ./deploy.sh mainnet all        # Deploy to all chains on mainnet"
    exit 0
fi

main $1 $2