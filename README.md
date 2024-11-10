### SQLite Server in Rust using sqlx backend to manage db
Because it's fun ^^

Note: Will probably error since the db files I have tested with aren't included.

- API (so far)
    - Paths made for testing purposes (this logic will live in a consumer-side lib of some kind later on. For my use-case, a Deno TS Server side library for making requests according to pre-defined parameters towards the SQLite rust server.)
        - POST["/generate_token"]
            - cURL
                - 
                ```
                curl --location 'localhost:8080/generate_token' \
                --header 'u_: b1a74559bea16b1521205f95f07a25ea2f09f49eb4e265fa6057036d1dff7c22' \
                --header 'Content-Type: application/json' \
                --data '{
                    "base_query": "SELECT * FROM users WHERE username = ? AND first_name = ? AND age = ?;",
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
        - POST["/migrate/{database}"]
            - TBD
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
    - currently enclosed in a file (will probably use special db file specific to this config)
    - This will be managed by CLI binary (server keeps a Arc Mutex handle on a HashMap that will be populated on db file change) (not sure how to do this yet)
    - schema
        - 
        ```
        <username>|<username_hash>|<(username+pw)_hash>|<database>:<access_level>,<database>:<access_level>
        ```
    - access levels (think linux)
        - 0 nothing
        - 1 read
        - 2 write
        - 3 read + write
        - 4 not sure yet, possibly create / delete

        So assuming you have access to create + read + write, your access will be 7 (4 + (3 or 1 + 2, whichever way you wanna think about it))
