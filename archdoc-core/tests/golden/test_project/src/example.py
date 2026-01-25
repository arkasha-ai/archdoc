"""Example module for testing."""

import os
from typing import List

class Calculator:
    """A simple calculator class."""
    
    def __init__(self):
        """Initialize the calculator."""
        pass
    
    def add(self, a: int, b: int) -> int:
        """Add two numbers."""
        return a + b
    
    def multiply(self, a: int, b: int) -> int:
        """Multiply two numbers."""
        return a * b

def process_numbers(numbers: List[int]) -> List[int]:
    """Process a list of numbers."""
    calc = Calculator()
    return [calc.add(n, 1) for n in numbers]

if __name__ == "__main__":
    numbers = [1, 2, 3, 4, 5]
    result = process_numbers(numbers)
    print(f"Processed numbers: {result}")