CREATE TABLE IF NOT EXISTS posts(
    body TEXT NOT NULL,
    image BLOB,
    publish_date INTEGER NOT NULL,
    user_name TEXT NOT NULL,
    avatar BLOB
);