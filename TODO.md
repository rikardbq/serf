- [x] Add core functionality for write and read to db
- [x] fix the query function to allow more generic argument lists / partial application of query params
- [x] Handle different types of calls, I.E inserts vs fetches, etc (pass a subject in the token?)
- [x] Add web controllers in web lib
- [-] setup rest endpoints for actual use)
    - [x] "{database}" path more or less done
    - [] "{database}/migrate" (for running migrations)
        - scripts and management of these will have to be done on consumer side unless sqlx migrate has some feature for keeping track. Possibly both are needed to avoid calling the server when not needed.
- [-] start work on the CLI for adding users to user management db
    - basic idea is to manage users with add, remove, modify commands (may have to use some library to manage sub-commanding)
    - same pattern would apply for managing DB's and user access to DB's (not final)
        - ```sqlite_server_cli add user -u rikardbq -p somepass```
        - ```sqlite_server_cli remove user -u rikardbq```
        - ```sqlite_server_cli modify user chpass -u rikardbq -op oldpass -np newpass```
- [] handle updates to the user management db so that the Arc handle gets the latest user hashmap
    - use "notify" crate to listen on the db file change?
- [] add support for config properties file + build step arguments
    - sqlite_server_config_path=$HOME/.local/usr/.sqlite_server/ or %APPDATA_LOCAL%/.sqlite_server/ depending on architecture built on
    - sqlite_server_db_path=$config_path/db/ ???
    - Possible solution:
        - use build.rs file with sane defaults
            - use app.properties file of some kind
                - use plain text and split on newline and then split args on " = "-sign
                - example: 
                    ```
                    sqlite_server_config_root={ROOT_CONF_LOCATION_PLACEHOLDER}
                    sqlite_server_binary_output={BINARY_OUTPUT_LOCATION_PLACEHOLDER}
                    ```
                - at build replace the values with either the sane defaults or provided args.
                    - folders will be created and populated with necessary files
                    - this app.properties file will then be read on startup and used throughout the application.
        - accept arguments to build step to manage any custom paths for config location and/or output binary location
        - if no arguments are supplied to build then the default values are preferred.
