#!/bin/bash

# CrossChain DSL Demo Environment Setup Script
# This script sets up a complete demo environment for showcasing the platform

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Functions
print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ️  $1${NC}"
}

# Check prerequisites
check_prerequisites() {
    print_info "Checking prerequisites..."
    
    # Check Rust
    if command -v cargo &> /dev/null; then
        print_success "Rust is installed"
    else
        print_error "Rust is not installed. Please install from https://rustup.rs/"
        exit 1
    fi
    
    # Check Node.js
    if command -v node &> /dev/null; then
        print_success "Node.js is installed"
    else
        print_error "Node.js is not installed. Please install Node.js 18+"
        exit 1
    fi
    
    # Check Python
    if command -v python3 &> /dev/null; then
        print_success "Python is installed"
    else
        print_error "Python is not installed. Please install Python 3.8+"
        exit 1
    fi
    
    # Check Docker (optional)
    if command -v docker &> /dev/null; then
        print_success "Docker is installed (optional)"
    else
        print_info "Docker is not installed. Some features may not work."
    fi
}

# Setup environment variables
setup_env() {
    print_info "Setting up environment variables..."
    
    if [ ! -f "ai-integration/.env" ]; then
        cp ai-integration/.env.example ai-integration/.env
        print_info "Created .env file. Please add your API keys:"
        print_info "  - OPENAI_API_KEY"
        print_info "  - ANTHROPIC_API_KEY"
    else
        print_success ".env file already exists"
    fi
}

# Build Rust projects
build_rust() {
    print_info "Building Rust projects..."
    
    # Build DSL compiler
    cd dsl-compiler
    cargo build --release
    print_success "DSL compiler built"
    cd ..
    
    # Build formal verification
    cd formal-verification
    cargo build --release
    print_success "Formal verification built"
    cd ..
    
    # Build LSP server
    cd lsp-server
    cargo build --release
    print_success "LSP server built"
    cd ..
}

# Setup Python environment
setup_python() {
    print_info "Setting up Python environment..."
    
    cd ai-integration
    
    # Create virtual environment
    python3 -m venv venv
    source venv/bin/activate
    
    # Install dependencies
    pip install -r requirements.txt
    print_success "Python dependencies installed"
    
    deactivate
    cd ..
}

# Setup Node.js projects
setup_node() {
    print_info "Setting up Node.js projects..."
    
    # Install VS Code extension dependencies
    if [ -d "vscode-extension" ]; then
        cd vscode-extension
        npm install
        print_success "VS Code extension dependencies installed"
        cd ..
    fi
}

# Create demo web interface
create_web_interface() {
    print_info "Creating web interface..."
    
    mkdir -p web
    cd web
    
    # Create package.json
    cat > package.json << 'EOF'
{
  "name": "crosschain-dsl-web",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "dev": "next dev",
    "build": "next build",
    "start": "next start"
  },
  "dependencies": {
    "next": "14.0.0",
    "react": "18.2.0",
    "react-dom": "18.2.0",
    "@monaco-editor/react": "^4.6.0",
    "axios": "^1.6.0"
  },
  "devDependencies": {
    "@types/node": "20.8.0",
    "@types/react": "18.2.0",
    "typescript": "5.2.0",
    "tailwindcss": "^3.3.0",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.4.0"
  }
}
EOF
    
    # Create basic Next.js app structure
    mkdir -p pages api public styles
    
    # Create main page
    cat > pages/index.tsx << 'EOF'
import { useState } from 'react';
import Editor from '@monaco-editor/react';

export default function Home() {
  const [code, setCode] = useState(`contract MyToken {
    state {
        total_supply: u64;
        balances: map<address, u64>;
    }
    
    public fn transfer(to: address, amount: u64) {
        require(balances[msg_sender()] >= amount);
        balances[msg_sender()] -= amount;
        balances[to] += amount;
    }
}`);

  const [output, setOutput] = useState('');
  const [loading, setLoading] = useState(false);

  const compile = async () => {
    setLoading(true);
    try {
      const response = await fetch('/api/compile', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ code, targets: ['solana', 'aptos'] })
      });
      const result = await response.json();
      setOutput(JSON.stringify(result, null, 2));
    } catch (error) {
      setOutput(`Error: ${error.message}`);
    }
    setLoading(false);
  };

  return (
    <div className="min-h-screen bg-gray-900 text-white">
      <header className="bg-gray-800 p-4">
        <h1 className="text-2xl font-bold">🚀 CrossChain DSL Demo</h1>
        <p className="text-gray-400">Write once, deploy everywhere</p>
      </header>
      
      <div className="container mx-auto p-4">
        <div className="grid grid-cols-2 gap-4">
          <div>
            <h2 className="text-xl mb-2">DSL Code</h2>
            <Editor
              height="500px"
              defaultLanguage="rust"
              theme="vs-dark"
              value={code}
              onChange={(value) => setCode(value || '')}
            />
          </div>
          
          <div>
            <h2 className="text-xl mb-2">Compilation Output</h2>
            <pre className="bg-gray-800 p-4 rounded h-[500px] overflow-auto">
              {output || 'Click compile to see output'}
            </pre>
          </div>
        </div>
        
        <div className="mt-4 flex gap-4">
          <button
            onClick={compile}
            disabled={loading}
            className="bg-blue-600 hover:bg-blue-700 px-6 py-2 rounded disabled:opacity-50"
          >
            {loading ? 'Compiling...' : 'Compile'}
          </button>
          
          <button
            onClick={() => {/* AI generation */}}
            className="bg-green-600 hover:bg-green-700 px-6 py-2 rounded"
          >
            AI Generate
          </button>
          
          <button
            onClick={() => {/* Formal verification */}}
            className="bg-purple-600 hover:bg-purple-700 px-6 py-2 rounded"
          >
            Verify
          </button>
        </div>
      </div>
    </div>
  );
}
EOF
    
    # Create API endpoint
    cat > pages/api/compile.ts << 'EOF'
import type { NextApiRequest, NextApiResponse } from 'next';

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  if (req.method !== 'POST') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  const { code, targets } = req.body;

  // Call the actual compiler service
  try {
    // For demo, return mock response
    const result = {
      success: true,
      outputs: {
        solana: "// Generated Solana code\n" + code,
        aptos: "// Generated Aptos code\n" + code
      },
      metrics: {
        compilation_time: 234,
        code_reduction: 65
      }
    };
    
    res.status(200).json(result);
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
}
EOF
    
    # Install dependencies
    npm install
    print_success "Web interface created"
    
    cd ..
}

# Start demo services
start_services() {
    print_info "Starting demo services..."
    
    # Create start script
    cat > start-all.sh << 'EOF'
#!/bin/bash

# Start compiler service
echo "Starting compiler service..."
cd dsl-compiler && cargo run --release &
COMPILER_PID=$!

# Start AI service
echo "Starting AI service..."
cd ai-integration && source venv/bin/activate && python advanced_ai_engine.py serve &
AI_PID=$!

# Start web interface
echo "Starting web interface..."
cd web && npm run dev &
WEB_PID=$!

echo "Services started:"
echo "  Compiler: PID $COMPILER_PID"
echo "  AI Service: PID $AI_PID"
echo "  Web Interface: PID $WEB_PID"

echo ""
echo "🎉 Demo environment is running!"
echo "📍 Web Interface: http://localhost:3000"
echo ""
echo "Press Ctrl+C to stop all services"

# Wait for interrupt
trap "kill $COMPILER_PID $AI_PID $WEB_PID; exit" INT
wait
EOF
    
    chmod +x start-all.sh
    print_success "Start script created: ./start-all.sh"
}

# Create demo examples
create_examples() {
    print_info "Creating demo examples..."
    
    mkdir -p demo-examples
    
    # Create token example
    cat > demo-examples/token.ccdsl << 'EOF'
contract SimpleToken {
    state {
        name: string;
        symbol: string;
        total_supply: u64;
        balances: map<address, u64>;
    }
    
    public fn initialize(token_name: string, token_symbol: string, supply: u64) {
        name = token_name;
        symbol = token_symbol;
        total_supply = supply;
        balances[msg_sender()] = supply;
    }
    
    public fn transfer(to: address, amount: u64) {
        require(balances[msg_sender()] >= amount, "Insufficient balance");
        balances[msg_sender()] -= amount;
        balances[to] += amount;
        emit Transfer(msg_sender(), to, amount);
    }
    
    public fn balance_of(account: address) -> u64 {
        return balances[account];
    }
}
EOF
    
    # Create AMM example
    cat > demo-examples/amm.ccdsl << 'EOF'
contract SimpleAMM {
    state {
        token_a: address;
        token_b: address;
        reserve_a: u64;
        reserve_b: u64;
        lp_supply: u64;
        lp_balances: map<address, u64>;
    }
    
    public fn add_liquidity(amount_a: u64, amount_b: u64) -> u64 {
        let lp_amount = if lp_supply == 0 {
            sqrt(amount_a * amount_b)
        } else {
            min(
                amount_a * lp_supply / reserve_a,
                amount_b * lp_supply / reserve_b
            )
        };
        
        reserve_a += amount_a;
        reserve_b += amount_b;
        lp_supply += lp_amount;
        lp_balances[msg_sender()] += lp_amount;
        
        return lp_amount;
    }
    
    public fn swap(amount_in: u64, token_in: address) -> u64 {
        require(token_in == token_a || token_in == token_b, "Invalid token");
        
        let (reserve_in, reserve_out) = if token_in == token_a {
            (reserve_a, reserve_b)
        } else {
            (reserve_b, reserve_a)
        };
        
        let amount_out = (amount_in * 997 * reserve_out) / (reserve_in * 1000 + amount_in * 997);
        
        if token_in == token_a {
            reserve_a += amount_in;
            reserve_b -= amount_out;
        } else {
            reserve_b += amount_in;
            reserve_a -= amount_out;
        }
        
        return amount_out;
    }
}
EOF
    
    print_success "Demo examples created"
}

# Main execution
main() {
    echo "🚀 CrossChain DSL Demo Setup"
    echo "============================"
    echo ""
    
    check_prerequisites
    setup_env
    build_rust
    setup_python
    setup_node
    create_web_interface
    start_services
    create_examples
    
    echo ""
    print_success "Demo environment setup complete!"
    echo ""
    echo "📚 Next steps:"
    echo "  1. Add your API keys to ai-integration/.env"
    echo "  2. Run ./start-all.sh to start the demo"
    echo "  3. Open http://localhost:3000 in your browser"
    echo "  4. Try the examples in demo-examples/"
    echo ""
    echo "📹 Ready to record your demo video!"
}

# Run main function
main