pub const GET_USERS_AND_ACCESS: &str = r#"
    SELECT 
        u.username,
        u.username_password_hash,
        u.username_hash,
        json_array(
            json_object(
                database, access_right
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
        access_right INTEGER NOT NULL DEFAULT 1,
        username_hash TEXT NOT NULL,
        UNIQUE(database,username_hash)
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

pub const INSERT_USER_DATABASE_ACCESS: &str = r#"
    INSERT OR IGNORE INTO users_database_access(
        database,
        access_right,
        username_hash
    ) VALUES (?, ?, ?);
"#;
