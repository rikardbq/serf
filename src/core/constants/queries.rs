pub const GET_USERS_AND_ACCESS: &str = r#"
    SELECT 
        u.username,
        u.username_password_hash,
        u.username_hash,
        json_array(
            json_object(
                database_hash, access_right
            )
        ) as databases
    FROM users u INNER JOIN users_database_access USING(username_hash);
"#;

pub const CREATE_USERS_TABLE: &str = r#"
    CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY,
        username TEXT NOT NULL UNIQUE,
        username_hash TEXT NOT NULL UNIQUE,
        username_password_hash TEXT NOT NULL
    );
"#;

pub const CREATE_USERS_DATABASE_ACCESS_TABLE: &str = r#"
    CREATE TABLE IF NOT EXISTS users_database_access (
        id INTEGER PRIMARY KEY,
        database TEXT NOT NULL,
        database_hash TEXT NOT NULL,
        access_right INTEGER NOT NULL DEFAULT 1,
        username_hash TEXT NOT NULL,
        UNIQUE(database_hash,username_hash)
        FOREIGN KEY (username_hash)
        REFERENCES users (username_hash)
            ON UPDATE CASCADE
            ON DELETE CASCADE
    );
"#;

pub const INSERT_USER: &str = r#"
    INSERT OR IGNORE INTO users(
        username,
        username_hash,
        username_password_hash
    ) VALUES(?, ?, ?);
"#;

pub const UPSERT_USER_DATABASE_ACCESS: &str = r#"
    INSERT OR IGNORE INTO users_database_access(
        database,
        database_hash,
        access_right,
        username_hash
    ) VALUES(?, ?, ?, ?)
    ON CONFLICT(
        database_hash,
        username_hash
    ) DO UPDATE SET
        access_right = excluded.access_right
    WHERE excluded.database_hash=users_database_access.database_hash AND excluded.username_hash=users_database_access.username_hash;
"#;

pub const CREATE_MIGRATIONS_TABLE: &str = r#"
    CREATE TABLE IF NOT EXISTS __migrations_tracker_t__ (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        query TEXT NOT NULL
    );
    CREATE UNIQUE INDEX IF NOT EXISTS idx_name ON __migrations_tracker_t__ (name);
"#;

pub const INSERT_MIGRATION: &str = r#"
    INSERT INTO __migrations_tracker_t__(
        name,
        query
    ) VALUES(?, ?);
"#;
