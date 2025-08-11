#!/usr/bin/env python3
"""
CrossChain DSL Plugin Marketplace System
Handles plugin discovery, installation, and monetization
"""

import os
import json
import hashlib
import asyncio
from typing import Dict, List, Optional, Any
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
import aiohttp
from pydantic import BaseModel, Field

class PluginCategory(Enum):
    DEFI = "defi"
    NFT = "nft"
    DAO = "dao"
    SECURITY = "security"
    OPTIMIZATION = "optimization"
    INTEGRATION = "integration"
    CHAIN = "chain"
    TESTING = "testing"
    UTILITY = "utility"

class PricingModel(Enum):
    FREE = "free"
    ONE_TIME = "one_time"
    SUBSCRIPTION = "subscription"
    PAY_PER_USE = "pay_per_use"
    FREEMIUM = "freemium"

class PluginStatus(Enum):
    DRAFT = "draft"
    IN_REVIEW = "in_review"
    APPROVED = "approved"
    REJECTED = "rejected"
    PUBLISHED = "published"
    DEPRECATED = "deprecated"

@dataclass
class PluginMetadata:
    """Plugin metadata and manifest"""
    id: str
    name: str
    version: str
    description: str
    author: str
    category: PluginCategory
    tags: List[str]
    icon_url: Optional[str] = None
    repository_url: Optional[str] = None
    documentation_url: Optional[str] = None
    license: str = "MIT"
    created_at: datetime = field(default_factory=datetime.now)
    updated_at: datetime = field(default_factory=datetime.now)
    
    def to_dict(self) -> Dict:
        return {
            "id": self.id,
            "name": self.name,
            "version": self.version,
            "description": self.description,
            "author": self.author,
            "category": self.category.value,
            "tags": self.tags,
            "icon_url": self.icon_url,
            "repository_url": self.repository_url,
            "documentation_url": self.documentation_url,
            "license": self.license,
            "created_at": self.created_at.isoformat(),
            "updated_at": self.updated_at.isoformat(),
        }

@dataclass
class PluginPricing:
    """Plugin pricing information"""
    model: PricingModel
    price: float = 0.0  # In USD
    currency: str = "USD"
    subscription_period: Optional[str] = None  # monthly, yearly
    usage_unit: Optional[str] = None  # for pay-per-use
    trial_period_days: int = 0
    
    def calculate_price(self, usage: Optional[int] = None) -> float:
        if self.model == PricingModel.FREE:
            return 0.0
        elif self.model == PricingModel.ONE_TIME:
            return self.price
        elif self.model == PricingModel.SUBSCRIPTION:
            return self.price
        elif self.model == PricingModel.PAY_PER_USE and usage:
            return self.price * usage
        return self.price

@dataclass
class PluginStats:
    """Plugin usage and rating statistics"""
    downloads: int = 0
    active_installs: int = 0
    rating: float = 0.0
    reviews_count: int = 0
    revenue_total: float = 0.0
    revenue_last_month: float = 0.0
    
class Plugin(BaseModel):
    """Complete plugin representation"""
    metadata: Dict
    pricing: Dict
    stats: Dict
    status: str
    code_hash: str
    dependencies: List[str] = Field(default_factory=list)
    compatible_versions: List[str] = Field(default_factory=list)
    
    class Config:
        arbitrary_types_allowed = True

class PluginMarketplace:
    """Main marketplace system"""
    
    def __init__(self, marketplace_url: str = "https://marketplace.crosschain-dsl.io"):
        self.marketplace_url = marketplace_url
        self.plugins: Dict[str, Plugin] = {}
        self.user_purchases: Dict[str, List[str]] = {}
        self.developer_plugins: Dict[str, List[str]] = {}
        self.revenue_pool: float = 0.0
        
    async def publish_plugin(
        self,
        developer_id: str,
        metadata: PluginMetadata,
        pricing: PluginPricing,
        plugin_code: str,
        dependencies: List[str] = None
    ) -> Plugin:
        """Publish a new plugin to the marketplace"""
        
        # Generate unique plugin ID
        plugin_id = self._generate_plugin_id(metadata.name, metadata.version)
        metadata.id = plugin_id
        
        # Calculate code hash for integrity
        code_hash = hashlib.sha256(plugin_code.encode()).hexdigest()
        
        # Create plugin object
        plugin = Plugin(
            metadata=metadata.to_dict(),
            pricing=pricing.__dict__,
            stats=PluginStats().__dict__,
            status=PluginStatus.IN_REVIEW.value,
            code_hash=code_hash,
            dependencies=dependencies or [],
            compatible_versions=["1.0.0", "1.1.0", "1.2.0"]
        )
        
        # Store plugin
        self.plugins[plugin_id] = plugin
        
        # Track developer's plugins
        if developer_id not in self.developer_plugins:
            self.developer_plugins[developer_id] = []
        self.developer_plugins[developer_id].append(plugin_id)
        
        # Trigger review process
        await self._submit_for_review(plugin)
        
        return plugin
    
    async def search_plugins(
        self,
        query: Optional[str] = None,
        category: Optional[PluginCategory] = None,
        tags: Optional[List[str]] = None,
        pricing_model: Optional[PricingModel] = None,
        sort_by: str = "downloads"
    ) -> List[Plugin]:
        """Search and filter plugins"""
        
        results = []
        
        for plugin_id, plugin in self.plugins.items():
            # Filter by status
            if plugin.status != PluginStatus.PUBLISHED.value:
                continue
            
            # Filter by category
            if category and plugin.metadata["category"] != category.value:
                continue
            
            # Filter by pricing model
            if pricing_model and plugin.pricing["model"] != pricing_model.value:
                continue
            
            # Filter by tags
            if tags:
                plugin_tags = set(plugin.metadata.get("tags", []))
                if not any(tag in plugin_tags for tag in tags):
                    continue
            
            # Search in name and description
            if query:
                query_lower = query.lower()
                name_match = query_lower in plugin.metadata["name"].lower()
                desc_match = query_lower in plugin.metadata["description"].lower()
                if not (name_match or desc_match):
                    continue
            
            results.append(plugin)
        
        # Sort results
        if sort_by == "downloads":
            results.sort(key=lambda p: p.stats["downloads"], reverse=True)
        elif sort_by == "rating":
            results.sort(key=lambda p: p.stats["rating"], reverse=True)
        elif sort_by == "price":
            results.sort(key=lambda p: p.pricing["price"])
        elif sort_by == "newest":
            results.sort(key=lambda p: p.metadata["created_at"], reverse=True)
        
        return results
    
    async def purchase_plugin(
        self,
        user_id: str,
        plugin_id: str,
        payment_method: str = "credit_card"
    ) -> Dict[str, Any]:
        """Process plugin purchase"""
        
        if plugin_id not in self.plugins:
            raise ValueError(f"Plugin {plugin_id} not found")
        
        plugin = self.plugins[plugin_id]
        pricing = PluginPricing(**plugin.pricing)
        
        # Check if already purchased
        if user_id in self.user_purchases and plugin_id in self.user_purchases[user_id]:
            return {
                "status": "already_owned",
                "message": "You already own this plugin"
            }
        
        # Process payment
        payment_result = await self._process_payment(
            user_id,
            pricing.calculate_price(),
            payment_method
        )
        
        if payment_result["success"]:
            # Record purchase
            if user_id not in self.user_purchases:
                self.user_purchases[user_id] = []
            self.user_purchases[user_id].append(plugin_id)
            
            # Update stats
            plugin.stats["downloads"] += 1
            plugin.stats["active_installs"] += 1
            
            # Calculate revenue split
            revenue = pricing.calculate_price()
            developer_share = revenue * 0.7  # 70% to developer
            platform_share = revenue * 0.3   # 30% to platform
            
            plugin.stats["revenue_total"] += revenue
            plugin.stats["revenue_last_month"] += revenue
            self.revenue_pool += platform_share
            
            # Generate download token
            download_token = self._generate_download_token(user_id, plugin_id)
            
            return {
                "status": "success",
                "message": "Plugin purchased successfully",
                "download_token": download_token,
                "installation_guide": self._get_installation_guide(plugin_id)
            }
        else:
            return {
                "status": "failed",
                "message": payment_result.get("error", "Payment failed")
            }
    
    async def install_plugin(
        self,
        user_id: str,
        plugin_id: str,
        project_path: str
    ) -> bool:
        """Install plugin to user's project"""
        
        # Verify ownership
        if not self._verify_ownership(user_id, plugin_id):
            raise PermissionError("You don't own this plugin")
        
        plugin = self.plugins[plugin_id]
        
        # Download plugin files
        plugin_files = await self._download_plugin_files(plugin_id)
        
        # Install dependencies
        for dep in plugin.dependencies:
            await self._install_dependency(dep, project_path)
        
        # Copy plugin files to project
        plugin_dir = os.path.join(project_path, "plugins", plugin.metadata["name"])
        os.makedirs(plugin_dir, exist_ok=True)
        
        for file_name, content in plugin_files.items():
            file_path = os.path.join(plugin_dir, file_name)
            with open(file_path, 'w') as f:
                f.write(content)
        
        # Update project configuration
        self._update_project_config(project_path, plugin)
        
        return True
    
    async def rate_plugin(
        self,
        user_id: str,
        plugin_id: str,
        rating: float,
        review: Optional[str] = None
    ) -> bool:
        """Rate and review a plugin"""
        
        # Verify ownership
        if not self._verify_ownership(user_id, plugin_id):
            raise PermissionError("You must own the plugin to rate it")
        
        plugin = self.plugins[plugin_id]
        
        # Update rating
        current_rating = plugin.stats["rating"]
        review_count = plugin.stats["reviews_count"]
        
        # Calculate new average rating
        new_rating = ((current_rating * review_count) + rating) / (review_count + 1)
        
        plugin.stats["rating"] = round(new_rating, 2)
        plugin.stats["reviews_count"] += 1
        
        # Store review if provided
        if review:
            await self._store_review(plugin_id, user_id, rating, review)
        
        return True
    
    async def get_developer_dashboard(self, developer_id: str) -> Dict[str, Any]:
        """Get developer dashboard data"""
        
        if developer_id not in self.developer_plugins:
            return {
                "plugins": [],
                "total_revenue": 0,
                "total_downloads": 0,
                "average_rating": 0
            }
        
        plugins_data = []
        total_revenue = 0
        total_downloads = 0
        total_rating = 0
        rating_count = 0
        
        for plugin_id in self.developer_plugins[developer_id]:
            plugin = self.plugins[plugin_id]
            
            plugins_data.append({
                "id": plugin_id,
                "name": plugin.metadata["name"],
                "downloads": plugin.stats["downloads"],
                "revenue": plugin.stats["revenue_total"],
                "rating": plugin.stats["rating"],
                "status": plugin.status
            })
            
            total_revenue += plugin.stats["revenue_total"] * 0.7  # Developer's 70% share
            total_downloads += plugin.stats["downloads"]
            
            if plugin.stats["reviews_count"] > 0:
                total_rating += plugin.stats["rating"]
                rating_count += 1
        
        average_rating = total_rating / rating_count if rating_count > 0 else 0
        
        return {
            "plugins": plugins_data,
            "total_revenue": round(total_revenue, 2),
            "total_downloads": total_downloads,
            "average_rating": round(average_rating, 2),
            "pending_payout": self._calculate_pending_payout(developer_id)
        }
    
    # Helper methods
    
    def _generate_plugin_id(self, name: str, version: str) -> str:
        """Generate unique plugin ID"""
        base = f"{name.lower().replace(' ', '-')}-{version}"
        timestamp = datetime.now().strftime("%Y%m%d%H%M%S")
        return f"{base}-{timestamp}"
    
    async def _submit_for_review(self, plugin: Plugin) -> None:
        """Submit plugin for review process"""
        # In production, this would trigger automated tests and manual review
        # For now, auto-approve after basic checks
        await asyncio.sleep(1)  # Simulate review time
        
        # Basic validation
        if plugin.metadata["name"] and plugin.metadata["description"]:
            plugin.status = PluginStatus.PUBLISHED.value
        else:
            plugin.status = PluginStatus.REJECTED.value
    
    async def _process_payment(
        self,
        user_id: str,
        amount: float,
        payment_method: str
    ) -> Dict[str, Any]:
        """Process payment (mock implementation)"""
        # In production, integrate with payment providers (Stripe, PayPal, crypto)
        
        # Mock successful payment
        return {
            "success": True,
            "transaction_id": f"txn_{hashlib.md5(f'{user_id}{amount}'.encode()).hexdigest()[:12]}",
            "amount": amount,
            "currency": "USD"
        }
    
    def _generate_download_token(self, user_id: str, plugin_id: str) -> str:
        """Generate secure download token"""
        data = f"{user_id}:{plugin_id}:{datetime.now().isoformat()}"
        return hashlib.sha256(data.encode()).hexdigest()
    
    def _get_installation_guide(self, plugin_id: str) -> str:
        """Get installation instructions for plugin"""
        return f"""
# Installation Guide for {plugin_id}

1. Ensure CrossChain DSL CLI is installed:
   ```bash
   npm install -g crosschain-dsl
   ```

2. Install the plugin:
   ```bash
   ccdsl plugin install {plugin_id}
   ```

3. Configure in your project:
   ```yaml
   # crosschain.config.yaml
   plugins:
     - {plugin_id}
   ```

4. Import and use in your contracts:
   ```dsl
   import "{plugin_id}"
   
   contract MyContract {{
     // Use plugin features
   }}
   ```
"""
    
    def _verify_ownership(self, user_id: str, plugin_id: str) -> bool:
        """Verify user owns the plugin"""
        return (
            user_id in self.user_purchases and
            plugin_id in self.user_purchases[user_id]
        )
    
    async def _download_plugin_files(self, plugin_id: str) -> Dict[str, str]:
        """Download plugin files from storage"""
        # Mock implementation - return sample files
        return {
            "index.ccdsl": f"// Plugin: {plugin_id}\n// Main plugin code",
            "manifest.json": json.dumps({
                "id": plugin_id,
                "version": "1.0.0",
                "main": "index.ccdsl"
            }),
            "README.md": f"# Plugin {plugin_id}\n\nPlugin documentation"
        }
    
    async def _install_dependency(self, dependency: str, project_path: str) -> None:
        """Install plugin dependency"""
        # Mock implementation
        print(f"Installing dependency: {dependency}")
    
    def _update_project_config(self, project_path: str, plugin: Plugin) -> None:
        """Update project configuration to include plugin"""
        config_path = os.path.join(project_path, "crosschain.config.yaml")
        # In production, properly parse and update YAML
        print(f"Updated project config with plugin: {plugin.metadata['name']}")
    
    async def _store_review(
        self,
        plugin_id: str,
        user_id: str,
        rating: float,
        review: str
    ) -> None:
        """Store user review"""
        # In production, store in database
        print(f"Stored review for {plugin_id} by {user_id}: {rating} stars")
    
    def _calculate_pending_payout(self, developer_id: str) -> float:
        """Calculate pending payout for developer"""
        # In production, track actual payouts and calculate pending amount
        total = 0
        for plugin_id in self.developer_plugins.get(developer_id, []):
            plugin = self.plugins[plugin_id]
            total += plugin.stats["revenue_last_month"] * 0.7
        return round(total, 2)


# CLI Interface
async def main():
    """CLI for plugin marketplace"""
    import sys
    
    if len(sys.argv) < 2:
        print("""Usage: python plugin_system.py <command> [options]
        
Commands:
  publish <name> <version> <price> - Publish a new plugin
  search <query> - Search for plugins
  install <plugin_id> - Install a plugin
  dashboard <developer_id> - View developer dashboard
""")
        return
    
    marketplace = PluginMarketplace()
    command = sys.argv[1]
    
    if command == "publish":
        if len(sys.argv) < 5:
            print("Usage: publish <name> <version> <price>")
            return
        
        name = sys.argv[2]
        version = sys.argv[3]
        price = float(sys.argv[4])
        
        metadata = PluginMetadata(
            id="",
            name=name,
            version=version,
            description=f"Plugin {name} for CrossChain DSL",
            author="developer@example.com",
            category=PluginCategory.UTILITY,
            tags=["utility", "helper"]
        )
        
        pricing = PluginPricing(
            model=PricingModel.ONE_TIME if price > 0 else PricingModel.FREE,
            price=price
        )
        
        plugin = await marketplace.publish_plugin(
            "dev123",
            metadata,
            pricing,
            "// Plugin code here"
        )
        
        print(f"Plugin published: {plugin.metadata['id']}")
        print(f"Status: {plugin.status}")
    
    elif command == "search":
        query = sys.argv[2] if len(sys.argv) > 2 else None
        results = await marketplace.search_plugins(query=query)
        
        print(f"Found {len(results)} plugins:")
        for plugin in results:
            print(f"- {plugin.metadata['name']} v{plugin.metadata['version']}")
            print(f"  Price: ${plugin.pricing['price']}")
            print(f"  Downloads: {plugin.stats['downloads']}")
            print(f"  Rating: {plugin.stats['rating']}⭐")
    
    elif command == "dashboard":
        developer_id = sys.argv[2] if len(sys.argv) > 2 else "dev123"
        dashboard = await marketplace.get_developer_dashboard(developer_id)
        
        print("Developer Dashboard")
        print("=" * 40)
        print(f"Total Revenue: ${dashboard['total_revenue']}")
        print(f"Total Downloads: {dashboard['total_downloads']}")
        print(f"Average Rating: {dashboard['average_rating']}⭐")
        print(f"Pending Payout: ${dashboard['pending_payout']}")
        print("\nPlugins:")
        for plugin in dashboard['plugins']:
            print(f"- {plugin['name']}: {plugin['downloads']} downloads, ${plugin['revenue']} revenue")


if __name__ == "__main__":
    asyncio.run(main())