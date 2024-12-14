- [x] Add core functionality for write and read to db
- [x] fix the query function to allow more generic argument lists / partial application of query params
- [x] Handle different types of calls, I.E inserts vs fetches, etc (pass a subject in the token?)
- [x] Add web controllers in web lib
- [x] setup rest endpoints for actual use
    - [x] "{database}" more or less done
    - [x] "{database}/m" (for running migrations)
        - use special table for book keeping of migrations. "dbName.<table>" as "dbName.\_\_migrations_tracker_t\_\_"
        - versioning can be determined with chrono crate I.E datetime where 20241126180330 is YYYYmmddHHMMSS format
        - migrations will be of similar type as mutation requests
        - consumer side will bundle together all migrations that are still not applied
        - expectation at migration endpoint is that there are possibly many queries to be applied
- [-] start work on the CLI for adding users to user management db
    - [-] basic idea is to manage users with add, remove, modify commands (may have to use some library to manage sub-commanding)
    - same pattern would apply for managing DB's and user access to DB's (not final)
        - ```sqlite_server_cli add user -u rikardbq -p somepass```
        - ```sqlite_server_cli remove user -u rikardbq```
        - ```sqlite_server_cli modify user chpass -u rikardbq -op oldpass -np newpass```
- [] handle updates to the user management db so that the Arc handle gets the latest user hashmap
    - use "notify" crate to listen on the db file change?
- [x] root_dir=$HOME/.serf/ or %APPDATA_LOCAL%/.serf/ depending on architecture built on
    - consumer_db_path=$root_dir/db/{hashed_db_name}/
        - 1 folder per db to better namespace them on the filesystem since SQLite adds a bunch of meta files when manipulating the DB
- [x] use build.rs file with sane defaults
    - SERF_ROOT_DIR env in build script, defaults to ./serf from the project root dir when in dev
        - defaults(arch dependent) in build scripts
            - [x] win: %APPDATA%\.serf
            - [] unix: $HOME/.serf
    - folders will be created and populated with necessary files
- [x] use transactions for mutations
- [x] change usage of name "base_query in dat claim" to "query"
- [] break out mutation and query logic into separate endpoints???
- [] use middleware to check headers
- [] CACHE queries
    - post-processing step, I.E storing the data in CACHE and any potential tokenization \
    of query is done in a separate thread as soon as possible. \
    Caller thread still returns data to the user without waiting on any caching step.
    - use papaya concurrent hashmap
        - use base64 encoded version of the query
            - SELECT * FROM users a LEFT JOIN something b ON b.name = a.name WHERE a.name = ?
                
                hashmap_1
                (base64-query-string) : struct { 
                    expires?: timestamp(can be updated),
                    data: JsonValue 
                }

                hashmap_2
                (table name) : [
                    (base64-query-string_1),
                    (base64-query-string_2),
                    (base64-query-string_3),
                    (base64-query-string_4)
                ]

                (on write check hashmap_2 if the table written to exists in the map)
                (if true then use the array it stores as a value and remove all the entries from hashmap_1 that matches the hashmap_2 value array items)


    
    - CACHE busting will be done as a post-processing step to a write operation
        - possibly by finding if the table written to is in any of the tokenized query keys for the database in question
---

### BRANCH MIGRATIONS
---
- [x] Set up migrations (  {database}/m  ) endpoint
- [x] Handle multiple migrations coming to endpoint as bundle
- [x] Enforce shape of migration entry from consumer-side
- [x] Create custom table for tracking migrations. \_\_migrations_tracker_t\_\_
- [x] Add migrations support to JS connector lib, I.E consumer-side tracking file + some form of migration verification step.
- [x] Apply migrations 1 by 1

### BRANCH WORK/DECLUTTER
---
- [x] Create utility functions to handle CLI args in a simpler way
- [x] Create utility functions for retreiving database connections, check user access
- [x] Simplify flow in database access controller with utility for asserting user access for given claims subject and generating response claims
- [x] Add custom error type for clearer error messaging and differentiation in places where the return type to consumer may matter

### BRANCH CLI (merged)
---
- [x] Move user table creation to build.rs
- [x] Use env for setting the root dir defaults
- [x] Create Windows BATCH build and install script
- [x] Use transaction for initial users and access rights DB create
- [x] Further flesh out the CLI args list (needs more work)
- [x] Update AppState to include consumer DB root path(needs protection against escaping folder structure)
- [x] Update to execute_query fn to handle db transaction Executor type
- [x] Handle NULL data type in fetch as JsonValue as this also seems to be the type that is returned from column on json_group/object aggregators
- [x] Add queries const exports to cleanup the areas where DB calls are static and not dependent on user input
- [x] Use the users and access rights DB when checking user auth in database call from consumer
- [x] Change the database path param to be a hashed variant of the db name
- [x] Fetch users and access rights at srv bin startup and put in AppState usr field as Arc