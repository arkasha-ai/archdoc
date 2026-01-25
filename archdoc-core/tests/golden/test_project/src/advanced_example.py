"""Advanced example module for testing with integrations."""

import requests
import sqlite3
import redis
from typing import List, Dict

class UserService:
    """A service for managing users with database integration."""
    
    def __init__(self, db_path: str = "users.db"):
        """Initialize the user service with database path."""
        self.db_path = db_path
        self._init_db()
    
    def _init_db(self):
        """Initialize the database."""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT UNIQUE NOT NULL
            )
        """)
        conn.commit()
        conn.close()
    
    def create_user(self, name: str, email: str) -> Dict:
        """Create a new user in the database."""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        cursor.execute(
            "INSERT INTO users (name, email) VALUES (?, ?)",
            (name, email)
        )
        user_id = cursor.lastrowid
        conn.commit()
        conn.close()
        
        return {"id": user_id, "name": name, "email": email}
    
    def get_user(self, user_id: int) -> Dict:
        """Get a user by ID from the database."""
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        cursor.execute("SELECT * FROM users WHERE id = ?", (user_id,))
        row = cursor.fetchone()
        conn.close()
        
        if row:
            return {"id": row[0], "name": row[1], "email": row[2]}
        return None

class NotificationService:
    """A service for sending notifications with queue integration."""
    
    def __init__(self, redis_url: str = "redis://localhost:6379"):
        """Initialize the notification service with Redis URL."""
        self.redis_client = redis.Redis.from_url(redis_url)
    
    def send_email_notification(self, user_id: int, message: str) -> bool:
        """Send an email notification by queuing it."""
        notification = {
            "user_id": user_id,
            "message": message,
            "type": "email"
        }
        
        # Push to Redis queue
        self.redis_client.lpush("notifications", str(notification))
        return True

def fetch_external_user_data(user_id: int) -> Dict:
    """Fetch user data from an external API."""
    response = requests.get(f"https://api.example.com/users/{user_id}")
    if response.status_code == 200:
        return response.json()
    return {}

def process_users(user_ids: List[int]) -> List[Dict]:
    """Process a list of users with various integrations."""
    # Database integration
    user_service = UserService()
    
    # Queue integration
    notification_service = NotificationService()
    
    results = []
    for user_id in user_ids:
        # Database operation
        user = user_service.get_user(user_id)
        if user:
            # External API integration
            external_data = fetch_external_user_data(user_id)
            user.update(external_data)
            
            # Queue operation
            notification_service.send_email_notification(
                user_id, 
                f"Processing user {user['name']}"
            )
            
            results.append(user)
    
    return results