-- create users table
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    fullname VARCHAR(64) NOT NULL,
    email VARCHAR(64) NOT NULL,
    -- hashed argon2 password
    password VARCHAR(64) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- create chat type: single, group, private_channel, public_channel
CREATE TYPE chat_type AS ENUM ('single', 'group', 'private_channel', 'public_channel');

-- create chat table
CREATE TABLE IF NOT EXISTS chats (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(128) NOT NULL UNIQUE,
    type chat_type NOT NULL,
    -- user id list
    members BIGINT[] NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS messages (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL,
    sender_id BIGINT NOT NULL,
    content TEXT NOT NULL,
    images TEXT[],
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (chat_id) REFERENCES chats(id),
    FOREIGN KEY (sender_id) REFERENCES users(id)
);

-- create index on chat_id for messages table
-- 适合找某个聊天的最近消息
CREATE INDEX IF NOT EXISTS chat_id_index ON messages(chat_id, created_at DESC);

-- create index for messages sender_id
CREATE INDEX IF NOT EXISTS sender_id_index ON messages(sender_id);
        