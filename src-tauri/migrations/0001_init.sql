CREATE TABLE oauth_tokens (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      access_token TEXT NOT NULL,
      refresh_token TEXT NOT NULL,
      created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);