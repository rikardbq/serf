### SQLite Server in Rust using sqlx backend to manage db
Because it's fun ^^

--Note: Will probably error since the db files I have tested with aren't included.--
Note: OUTDATED

- API
    - For testing and simplifying development
        - POST["/generate_token"]
            - cURL
                - 
                ```
                curl --location 'localhost:8080/generate_token' \
                --header 'u_: b1a74559bea16b1521205f95f07a25ea2f09f49eb4e265fa6057036d1dff7c22' \
                --header 'Content-Type: application/json' \
                --data '{
                    "query": "SELECT * FROM users WHERE username = ? AND first_name = ? AND age = ?;",
                    "parts": ["rikardbq", "Rikard", 35]
                }'
                ```
        - POST["/verify_token"]
            - cURL
                -
                ```
                curl --location 'localhost:8080/verify_token' \
                --header 'u_: b1a74559bea16b1521205f95f07a25ea2f09f49eb4e265fa6057036d1dff7c22' \
                --header 'Content-Type: application/json' \
                --data '{
                    "payload": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJzXyIsInN1YiI6ImRfIiwiYXVkIjoiY18iLCJkYXQiOiJ7XHJcbiAgICBcImJhc2VfcXVlcnlcIjogXCJTRUxFQ1QgKiBGUk9NIHVzZXJzNTtcIixcclxuICAgIFwicGFydHNcIjogW11cclxufSIsImlhdCI6MTczMTIzMjgzOCwiZXhwIjoxNzMxMjMyODY4fQ.W5AK92hsNhFGpJmgax7ylybwZGSIBueCVD-8J7YLNqg"
                }'
                ```
        - POST["/decode_token"]
            - cURL
                -
                ```
                curl --location 'localhost:8080/decode_token' \
                --header 'u_: b1a74559bea16b1521205f95f07a25ea2f09f49eb4e265fa6057036d1dff7c22' \
                --header 'Content-Type: application/json' \
                --data '{
                    "payload": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJzXyIsInN1YiI6ImRfIiwiYXVkIjoiY18iLCJkYXQiOiJ7XHJcbiAgICBcImJhc2VfcXVlcnlcIjogXCJTRUxFQ1QgKiBGUk9NIHVzZXJzNTtcIixcclxuICAgIFwicGFydHNcIjogW11cclxufSIsImlhdCI6MTczMTIzMjgzOCwiZXhwIjoxNzMxMjMyODY4fQ.W5AK92hsNhFGpJmgax7ylybwZGSIBueCVD-8J7YLNqg"
                }'
                ```

    - Server paths
        - POST["/{database}/m"]
            - Migration endpoint
                - possible solution will be keeping track of database migrations on the server and allowing database consumer to be "dumb" or just execute what comes in blindly and error.
                - consumer-side lib should have a notion of migrations, through a migration file pointing to schema changes in some way, either from a file or inline
        - POST["/{database}"]
            - cURL:
                - 
                ```
                curl --location 'localhost:8080/testing' \
                --header 'u_: b1a74559bea16b1521205f95f07a25ea2f09f49eb4e265fa6057036d1dff7c22' \
                --header 'Content-Type: application/json' \
                --data '{
                    "payload": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJzXyIsInN1YiI6ImRfIiwiYXVkIjoiY18iLCJkYXQiOiJ7XHJcbiAgICBcImJhc2VfcXVlcnlcIjogXCJTRUxFQ1QgKiBGUk9NIHVzZXJzNTtcIixcclxuICAgIFwicGFydHNcIjogW11cclxufSIsImlhdCI6MTczMTIzMjgzOCwiZXhwIjoxNzMxMjMyODY4fQ.W5AK92hsNhFGpJmgax7ylybwZGSIBueCVD-8J7YLNqg"
                }'
                ```

- User management
    - Create DB (flags= [-db \<db_name\>])
    ```
    cargo run --bin sqlite_server_cli -- create database -db testdb
    ```
    - Create User (flags= [-u \<username\>, -p \<password\>])
    ```
    cargo run --bin sqlite_server_cli -- create user -u rikardbq -p testpw123
    ```
    - Modify User
        - Access (flags= [-u \<username\>, -db \<db_name\>, -ar \<access-right(1=read, 2=write, 3=read+write)\>])
        ```
        cargo run --bin sqlite_server_cli -- modify user access -u rikardbq -db testdb -ar 3
        ```
        - access levels (think linux)
            - 0 nothing
            - 1 read
            - 2 write
            - 3 read + write
