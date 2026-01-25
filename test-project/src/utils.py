"""Utility functions for the test project."""

import json
import os

def load_config(config_path: str) -> dict:
    """Load configuration from a JSON file."""
    with open(config_path, 'r') as f:
        return json.load(f)

def save_config(config: dict, config_path: str):
    """Save configuration to a JSON file."""
    with open(config_path, 'w') as f:
        json.dump(config, f, indent=2)

def get_file_size(filepath: str) -> int:
    """Get the size of a file in bytes."""
    return os.path.getsize(filepath)

def format_bytes(size: int) -> str:
    """Format bytes into a human-readable string."""
    for unit in ['B', 'KB', 'MB', 'GB']:
        if size < 1024.0:
            return f"{size:.1f} {unit}"
        size /= 1024.0
    return f"{size:.1f} TB"