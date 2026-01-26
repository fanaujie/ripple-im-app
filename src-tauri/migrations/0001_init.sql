CREATE TABLE oauth_tokens (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      access_token TEXT NOT NULL,
      refresh_token TEXT NOT NULL,
      created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);


-- Application metadata (single row table)
CREATE TABLE IF NOT EXISTS app_metadata (
                                            id INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    device_id TEXT,
    user_id TEXT
    );

-- Ensure single row exists
INSERT OR IGNORE INTO app_metadata (id) VALUES (1);

-- User profile
CREATE TABLE IF NOT EXISTS user_profile (
                                            user_id TEXT PRIMARY KEY,
                                            nick_name TEXT NOT NULL,
                                            avatar TEXT
);

-- Relations (friends & blocked users)
CREATE TABLE IF NOT EXISTS relations (
                                         user_id TEXT PRIMARY KEY,
                                         nick_name TEXT NOT NULL,
                                         avatar TEXT,
                                         remark_name TEXT,
                                         relation_flags INTEGER NOT NULL
);

-- Relations sync version (single row table)
CREATE TABLE IF NOT EXISTS relations_version (
                                                 id INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    version TEXT
    );
INSERT OR IGNORE INTO relations_version (id) VALUES (1);

-- Conversations
CREATE TABLE IF NOT EXISTS conversations (
                                             conversation_id TEXT PRIMARY KEY,
                                             peer_id TEXT,
                                             group_id TEXT,
                                             last_message_id TEXT,
                                             last_read_message_id TEXT,
                                             unread_count INTEGER NOT NULL DEFAULT 0,
                                             last_message_text TEXT,
                                             last_message_timestamp INTEGER,
                                             name TEXT NOT NULL,
                                             avatar TEXT,
                                             bot_session_id TEXT
);

-- Conversations sync version (single row table)
CREATE TABLE IF NOT EXISTS conversations_version (
                                                     id INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    version TEXT
    );
INSERT OR IGNORE INTO conversations_version (id) VALUES (1);

-- Messages
CREATE TABLE IF NOT EXISTS messages (
                                        message_id TEXT PRIMARY KEY,
                                        conversation_id TEXT NOT NULL,
                                        sender_id TEXT NOT NULL,
                                        receiver_id TEXT,
                                        group_id TEXT,
                                        send_timestamp TEXT NOT NULL,
                                        message_type INTEGER NOT NULL,
                                        text TEXT,
                                        file_url TEXT,
                                        file_name TEXT,
                                        command_type INTEGER NOT NULL DEFAULT 0,
                                        command_data TEXT
);

-- User groups
CREATE TABLE IF NOT EXISTS user_groups (
                                           group_id TEXT PRIMARY KEY,
                                           group_name TEXT NOT NULL,
                                           group_avatar TEXT
);

-- User groups sync version (single row table)
CREATE TABLE IF NOT EXISTS user_groups_version (
                                                   id INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    version TEXT
    );
INSERT OR IGNORE INTO user_groups_version (id) VALUES (1);

-- Group members
CREATE TABLE IF NOT EXISTS group_members (
                                             group_id TEXT NOT NULL,
                                             user_id TEXT NOT NULL,
                                             name TEXT NOT NULL,
                                             avatar TEXT,
                                             PRIMARY KEY (group_id, user_id)
    );

-- Group member versions (one row per group)
CREATE TABLE IF NOT EXISTS group_member_versions (
                                                     group_id TEXT PRIMARY KEY,
                                                     version TEXT
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_messages_conversation_id ON messages(conversation_id);
CREATE INDEX IF NOT EXISTS idx_messages_conversation_timestamp ON messages(conversation_id, message_id DESC);
CREATE INDEX IF NOT EXISTS idx_group_members_group_id ON group_members(group_id);
