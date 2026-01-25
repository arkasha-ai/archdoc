"""Core module with database and HTTP integrations."""

import sqlite3
import requests

class DatabaseManager:
    """Manages database connections and operations."""
    
    def __init__(self, db_path: str):
        self.db_path = db_path
        self.connection = None
    
    def connect(self):
        """Connect to the database."""
        self.connection = sqlite3.connect(self.db_path)
    
    def execute_query(self, query: str):
        """Execute a database query."""
        if self.connection:
            cursor = self.connection.cursor()
            cursor.execute(query)
            return cursor.fetchall()

def fetch_external_data(url: str) -> dict:
    """Fetch data from an external API."""
    response = requests.get(url)
    return response.json()

def process_user_data(user_id: int) -> dict:
    """Process user data with database and external API calls."""
    # Database interaction
    db = DatabaseManager("users.db")
    db.connect()
    user_data = db.execute_query(f"SELECT * FROM users WHERE id = {user_id}")
    
    # External API call
    api_data = fetch_external_data(f"https://api.example.com/users/{user_id}")
    
    return {
        "user": user_data,
        "api": api_data
    }