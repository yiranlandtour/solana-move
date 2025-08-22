#!/usr/bin/env python3
"""
AI Configuration Management for CrossChain DSL
Handles API keys, model selection, and provider management
"""

import os
import json
import yaml
from pathlib import Path
from typing import Dict, Optional, List, Any
from dataclasses import dataclass, asdict
from enum import Enum
import logging

logger = logging.getLogger(__name__)

class AIProvider(Enum):
    """Supported AI providers"""
    OPENAI = "openai"
    ANTHROPIC = "anthropic"
    GOOGLE = "google"
    COHERE = "cohere"
    HUGGINGFACE = "huggingface"
    LOCAL = "local"

class ModelCapability(Enum):
    """Model capabilities"""
    CODE_GENERATION = "code_generation"
    CODE_REVIEW = "code_review"
    SECURITY_AUDIT = "security_audit"
    OPTIMIZATION = "optimization"
    DOCUMENTATION = "documentation"
    EXPLANATION = "explanation"
    TRANSLATION = "translation"

@dataclass
class ModelConfig:
    """Configuration for a specific AI model"""
    provider: AIProvider
    model_name: str
    max_tokens: int = 4000
    temperature: float = 0.7
    top_p: float = 0.95
    frequency_penalty: float = 0.0
    presence_penalty: float = 0.0
    capabilities: List[ModelCapability] = None
    cost_per_1k_tokens: float = 0.0
    rate_limit: int = 60  # requests per minute
    
    def __post_init__(self):
        if self.capabilities is None:
            self.capabilities = [ModelCapability.CODE_GENERATION]

@dataclass
class ProviderConfig:
    """Configuration for an AI provider"""
    name: AIProvider
    api_key: Optional[str] = None
    api_base: Optional[str] = None
    organization: Optional[str] = None
    models: Dict[str, ModelConfig] = None
    
    def __post_init__(self):
        if self.models is None:
            self.models = {}

class AIConfigManager:
    """Manages AI configuration and credentials"""
    
    DEFAULT_CONFIG_PATH = Path.home() / ".crosschain-dsl" / "ai_config.yaml"
    
    def __init__(self, config_path: Optional[Path] = None):
        self.config_path = config_path or self.DEFAULT_CONFIG_PATH
        self.providers: Dict[AIProvider, ProviderConfig] = {}
        self.active_models: Dict[ModelCapability, ModelConfig] = {}
        
        # Ensure config directory exists
        self.config_path.parent.mkdir(parents=True, exist_ok=True)
        
        # Load configuration
        self.load_config()
        
        # Load environment variables as fallback
        self.load_env_vars()
    
    def load_config(self) -> None:
        """Load configuration from file"""
        if self.config_path.exists():
            try:
                with open(self.config_path, 'r') as f:
                    config_data = yaml.safe_load(f)
                    
                if config_data:
                    self._parse_config(config_data)
                    logger.info(f"Loaded AI configuration from {self.config_path}")
            except Exception as e:
                logger.error(f"Failed to load config: {e}")
        else:
            logger.info("No configuration file found, using defaults")
            self._setup_default_config()
    
    def save_config(self) -> None:
        """Save current configuration to file"""
        config_data = {
            "providers": {},
            "active_models": {}
        }
        
        # Serialize providers
        for provider_name, provider_config in self.providers.items():
            config_data["providers"][provider_name.value] = {
                "api_key": provider_config.api_key,
                "api_base": provider_config.api_base,
                "organization": provider_config.organization,
                "models": {}
            }
            
            for model_name, model_config in provider_config.models.items():
                config_data["providers"][provider_name.value]["models"][model_name] = {
                    "max_tokens": model_config.max_tokens,
                    "temperature": model_config.temperature,
                    "top_p": model_config.top_p,
                    "capabilities": [cap.value for cap in model_config.capabilities],
                    "cost_per_1k_tokens": model_config.cost_per_1k_tokens,
                    "rate_limit": model_config.rate_limit
                }
        
        # Serialize active models
        for capability, model in self.active_models.items():
            config_data["active_models"][capability.value] = {
                "provider": model.provider.value,
                "model_name": model.model_name
            }
        
        with open(self.config_path, 'w') as f:
            yaml.dump(config_data, f, default_flow_style=False)
        
        logger.info(f"Saved AI configuration to {self.config_path}")
    
    def load_env_vars(self) -> None:
        """Load API keys from environment variables"""
        # OpenAI
        if openai_key := os.getenv("OPENAI_API_KEY"):
            if AIProvider.OPENAI not in self.providers:
                self.providers[AIProvider.OPENAI] = ProviderConfig(
                    name=AIProvider.OPENAI,
                    api_key=openai_key,
                    models=self._get_openai_models()
                )
            else:
                self.providers[AIProvider.OPENAI].api_key = openai_key
        
        # Anthropic
        if anthropic_key := os.getenv("ANTHROPIC_API_KEY"):
            if AIProvider.ANTHROPIC not in self.providers:
                self.providers[AIProvider.ANTHROPIC] = ProviderConfig(
                    name=AIProvider.ANTHROPIC,
                    api_key=anthropic_key,
                    models=self._get_anthropic_models()
                )
            else:
                self.providers[AIProvider.ANTHROPIC].api_key = anthropic_key
        
        # Google
        if google_key := os.getenv("GOOGLE_API_KEY"):
            if AIProvider.GOOGLE not in self.providers:
                self.providers[AIProvider.GOOGLE] = ProviderConfig(
                    name=AIProvider.GOOGLE,
                    api_key=google_key,
                    models=self._get_google_models()
                )
            else:
                self.providers[AIProvider.GOOGLE].api_key = google_key
    
    def _parse_config(self, config_data: Dict) -> None:
        """Parse configuration data"""
        # Parse providers
        if "providers" in config_data:
            for provider_name, provider_data in config_data["providers"].items():
                try:
                    provider_enum = AIProvider(provider_name)
                    provider_config = ProviderConfig(
                        name=provider_enum,
                        api_key=provider_data.get("api_key"),
                        api_base=provider_data.get("api_base"),
                        organization=provider_data.get("organization"),
                        models={}
                    )
                    
                    # Parse models
                    if "models" in provider_data:
                        for model_name, model_data in provider_data["models"].items():
                            capabilities = [
                                ModelCapability(cap) 
                                for cap in model_data.get("capabilities", ["code_generation"])
                            ]
                            
                            provider_config.models[model_name] = ModelConfig(
                                provider=provider_enum,
                                model_name=model_name,
                                max_tokens=model_data.get("max_tokens", 4000),
                                temperature=model_data.get("temperature", 0.7),
                                top_p=model_data.get("top_p", 0.95),
                                capabilities=capabilities,
                                cost_per_1k_tokens=model_data.get("cost_per_1k_tokens", 0.0),
                                rate_limit=model_data.get("rate_limit", 60)
                            )
                    
                    self.providers[provider_enum] = provider_config
                    
                except ValueError as e:
                    logger.warning(f"Unknown provider: {provider_name}")
        
        # Parse active models
        if "active_models" in config_data:
            for capability_name, model_ref in config_data["active_models"].items():
                try:
                    capability = ModelCapability(capability_name)
                    provider = AIProvider(model_ref["provider"])
                    
                    if provider in self.providers:
                        model_name = model_ref["model_name"]
                        if model_name in self.providers[provider].models:
                            self.active_models[capability] = self.providers[provider].models[model_name]
                
                except ValueError as e:
                    logger.warning(f"Invalid active model configuration: {e}")
    
    def _setup_default_config(self) -> None:
        """Setup default configuration"""
        # OpenAI defaults
        self.providers[AIProvider.OPENAI] = ProviderConfig(
            name=AIProvider.OPENAI,
            api_base="https://api.openai.com/v1",
            models=self._get_openai_models()
        )
        
        # Anthropic defaults
        self.providers[AIProvider.ANTHROPIC] = ProviderConfig(
            name=AIProvider.ANTHROPIC,
            api_base="https://api.anthropic.com/v1",
            models=self._get_anthropic_models()
        )
        
        # Set default active models
        if AIProvider.OPENAI in self.providers:
            gpt4_model = self.providers[AIProvider.OPENAI].models.get("gpt-4-turbo-preview")
            if gpt4_model:
                self.active_models[ModelCapability.CODE_GENERATION] = gpt4_model
                self.active_models[ModelCapability.CODE_REVIEW] = gpt4_model
                self.active_models[ModelCapability.SECURITY_AUDIT] = gpt4_model
    
    def _get_openai_models(self) -> Dict[str, ModelConfig]:
        """Get default OpenAI models"""
        return {
            "gpt-4-turbo-preview": ModelConfig(
                provider=AIProvider.OPENAI,
                model_name="gpt-4-turbo-preview",
                max_tokens=4096,
                temperature=0.7,
                capabilities=[
                    ModelCapability.CODE_GENERATION,
                    ModelCapability.CODE_REVIEW,
                    ModelCapability.SECURITY_AUDIT,
                    ModelCapability.OPTIMIZATION,
                    ModelCapability.DOCUMENTATION,
                    ModelCapability.EXPLANATION
                ],
                cost_per_1k_tokens=0.01,
                rate_limit=60
            ),
            "gpt-4": ModelConfig(
                provider=AIProvider.OPENAI,
                model_name="gpt-4",
                max_tokens=8192,
                temperature=0.7,
                capabilities=[
                    ModelCapability.CODE_GENERATION,
                    ModelCapability.CODE_REVIEW,
                    ModelCapability.SECURITY_AUDIT
                ],
                cost_per_1k_tokens=0.03,
                rate_limit=40
            ),
            "gpt-3.5-turbo": ModelConfig(
                provider=AIProvider.OPENAI,
                model_name="gpt-3.5-turbo",
                max_tokens=4096,
                temperature=0.7,
                capabilities=[
                    ModelCapability.CODE_GENERATION,
                    ModelCapability.DOCUMENTATION,
                    ModelCapability.EXPLANATION
                ],
                cost_per_1k_tokens=0.001,
                rate_limit=90
            )
        }
    
    def _get_anthropic_models(self) -> Dict[str, ModelConfig]:
        """Get default Anthropic models"""
        return {
            "claude-3-opus-20240229": ModelConfig(
                provider=AIProvider.ANTHROPIC,
                model_name="claude-3-opus-20240229",
                max_tokens=4096,
                temperature=0.7,
                capabilities=[
                    ModelCapability.CODE_GENERATION,
                    ModelCapability.CODE_REVIEW,
                    ModelCapability.SECURITY_AUDIT,
                    ModelCapability.OPTIMIZATION,
                    ModelCapability.DOCUMENTATION
                ],
                cost_per_1k_tokens=0.015,
                rate_limit=50
            ),
            "claude-3-sonnet-20240229": ModelConfig(
                provider=AIProvider.ANTHROPIC,
                model_name="claude-3-sonnet-20240229",
                max_tokens=4096,
                temperature=0.7,
                capabilities=[
                    ModelCapability.CODE_GENERATION,
                    ModelCapability.DOCUMENTATION,
                    ModelCapability.EXPLANATION
                ],
                cost_per_1k_tokens=0.003,
                rate_limit=70
            )
        }
    
    def _get_google_models(self) -> Dict[str, ModelConfig]:
        """Get default Google models"""
        return {
            "gemini-pro": ModelConfig(
                provider=AIProvider.GOOGLE,
                model_name="gemini-pro",
                max_tokens=8192,
                temperature=0.7,
                capabilities=[
                    ModelCapability.CODE_GENERATION,
                    ModelCapability.CODE_REVIEW,
                    ModelCapability.DOCUMENTATION
                ],
                cost_per_1k_tokens=0.00025,
                rate_limit=60
            )
        }
    
    def get_model_for_capability(self, capability: ModelCapability) -> Optional[ModelConfig]:
        """Get the active model for a specific capability"""
        return self.active_models.get(capability)
    
    def set_model_for_capability(
        self, 
        capability: ModelCapability, 
        provider: AIProvider, 
        model_name: str
    ) -> bool:
        """Set the active model for a specific capability"""
        if provider in self.providers:
            if model_name in self.providers[provider].models:
                model = self.providers[provider].models[model_name]
                if capability in model.capabilities:
                    self.active_models[capability] = model
                    return True
                else:
                    logger.warning(f"Model {model_name} doesn't support {capability.value}")
        return False
    
    def get_api_key(self, provider: AIProvider) -> Optional[str]:
        """Get API key for a provider"""
        if provider in self.providers:
            return self.providers[provider].api_key
        return None
    
    def set_api_key(self, provider: AIProvider, api_key: str) -> None:
        """Set API key for a provider"""
        if provider not in self.providers:
            self.providers[provider] = ProviderConfig(name=provider)
        self.providers[provider].api_key = api_key
    
    def list_available_models(self) -> Dict[AIProvider, List[str]]:
        """List all available models by provider"""
        result = {}
        for provider, config in self.providers.items():
            if config.api_key:  # Only list if API key is configured
                result[provider] = list(config.models.keys())
        return result
    
    def estimate_cost(
        self, 
        tokens: int, 
        capability: ModelCapability
    ) -> Optional[float]:
        """Estimate cost for a specific number of tokens"""
        if model := self.get_model_for_capability(capability):
            return (tokens / 1000) * model.cost_per_1k_tokens
        return None
    
    def validate_configuration(self) -> Dict[str, Any]:
        """Validate the current configuration"""
        issues = []
        warnings = []
        
        # Check for API keys
        configured_providers = []
        for provider, config in self.providers.items():
            if config.api_key:
                configured_providers.append(provider.value)
            else:
                warnings.append(f"No API key configured for {provider.value}")
        
        if not configured_providers:
            issues.append("No AI providers configured with API keys")
        
        # Check active models
        for capability in ModelCapability:
            if capability not in self.active_models:
                warnings.append(f"No model configured for {capability.value}")
        
        return {
            "valid": len(issues) == 0,
            "issues": issues,
            "warnings": warnings,
            "configured_providers": configured_providers,
            "active_capabilities": [cap.value for cap in self.active_models.keys()]
        }

# CLI tool for configuration management
def main():
    """CLI for managing AI configuration"""
    import argparse
    
    parser = argparse.ArgumentParser(description="AI Configuration Manager")
    subparsers = parser.add_subparsers(dest="command", help="Commands")
    
    # Show config
    show_parser = subparsers.add_parser("show", help="Show current configuration")
    
    # Set API key
    set_key_parser = subparsers.add_parser("set-key", help="Set API key")
    set_key_parser.add_argument("provider", choices=[p.value for p in AIProvider])
    set_key_parser.add_argument("api_key", help="API key")
    
    # Set model
    set_model_parser = subparsers.add_parser("set-model", help="Set model for capability")
    set_model_parser.add_argument("capability", choices=[c.value for c in ModelCapability])
    set_model_parser.add_argument("provider", choices=[p.value for p in AIProvider])
    set_model_parser.add_argument("model", help="Model name")
    
    # Validate
    validate_parser = subparsers.add_parser("validate", help="Validate configuration")
    
    # List models
    list_parser = subparsers.add_parser("list", help="List available models")
    
    args = parser.parse_args()
    
    manager = AIConfigManager()
    
    if args.command == "show":
        print("=== AI Configuration ===\n")
        
        print("Configured Providers:")
        for provider, config in manager.providers.items():
            key_status = "✓" if config.api_key else "✗"
            print(f"  {provider.value}: {key_status} (API key {'set' if config.api_key else 'not set'})")
        
        print("\nActive Models:")
        for capability, model in manager.active_models.items():
            print(f"  {capability.value}: {model.provider.value}/{model.model_name}")
    
    elif args.command == "set-key":
        provider = AIProvider(args.provider)
        manager.set_api_key(provider, args.api_key)
        manager.save_config()
        print(f"API key set for {args.provider}")
    
    elif args.command == "set-model":
        capability = ModelCapability(args.capability)
        provider = AIProvider(args.provider)
        
        if manager.set_model_for_capability(capability, provider, args.model):
            manager.save_config()
            print(f"Model set: {args.capability} -> {args.provider}/{args.model}")
        else:
            print(f"Failed to set model (check if model supports {args.capability})")
    
    elif args.command == "validate":
        validation = manager.validate_configuration()
        
        if validation["valid"]:
            print("✓ Configuration is valid")
        else:
            print("✗ Configuration has issues:")
            for issue in validation["issues"]:
                print(f"  - {issue}")
        
        if validation["warnings"]:
            print("\nWarnings:")
            for warning in validation["warnings"]:
                print(f"  ⚠ {warning}")
    
    elif args.command == "list":
        models = manager.list_available_models()
        
        if models:
            print("=== Available Models ===\n")
            for provider, model_list in models.items():
                print(f"{provider.value}:")
                for model in model_list:
                    print(f"  - {model}")
        else:
            print("No models available (configure API keys first)")

if __name__ == "__main__":
    main()