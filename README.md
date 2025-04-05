## SQLite Server in Rust using Actix Web and SQLx
#### Server to allow SQLite databases to serve its data over HTTP.
- To interface with it see **[serf-connector-java](https://github.com/rikardbq/serf-connector-java)** for Java applications, or **[serf-connector-deno](https://github.com/rikardbq/serf-connector-deno)** for Deno Javascript applications.

---

### Usage

#### CLI
**[ create - database ]**
- This command will create a db with the given name and will subsequently hash the db name to a url/file system friendly sha256 hex string.
- A db is located in your serf root dir db folder which by default will be ```$HOME/.serf/db/<db_hash>/<db_hash>.db``` accompanied by a file that only serves as a reminder to what the actual db name was should you forget it. (:

Example:
```
$ ./serf-cli create database -db <db_name>
```

**[ create - user ]**
- This command will create a user that the server can link to a database.
- The user is stored in the serf root dir cfg folder which by default will be ```$HOME/.serf/cfg/8d2394ce9279fee08d05ba52c882c6ca665b810fbdbf0cbc8ebe4a41364f7c11/8d2394ce9279fee08d05ba52c882c6ca665b810fbdbf0cbc8ebe4a41364f7c11.db```
- This hash is based on a static value in the build.rs file.

Example:
```
$ ./serf-cli create user -u <username> -p <password>
```

**[ modify - user - access ]**
- This command is used to link the user to a db together with an access right
- Access rights are integers from 1-3:
    - Where 1 = READ, 2 = WRITE, 3 = READ+WRITE

Example:
```
$ ./serf-cli modify user access -u <username> -db <db_name> -ar <access_right>
```

#### NOTE:
- A great tool for exploring and modifying SQLite databases that I use is [DB Browser for SQLite](https://sqlitebrowser.org/)

---

#### SERVER
**[ run the server ]**
- Optional arguments are:
    - --port \<number\> (default value: 8080)
    - --db-max-conn \<number\> (default value: 12)
    - --db-max-idle-time \<number_in_seconds\> (default value: 3600)

Flags explained:
- port
```
The network port that the application accepts connections on.
```
- db-max-conn
```
Set the maximum number of connections that this pool should maintain.
```
- db-max-idle-time
```
Set a maximum idle duration for individual connections.
Any connection that remains in the idle queue longer than this will be closed.
```

Example:
```
$ ./serf --port 8080 --db-max-conn 12 --db-max-idle-time 3600
```
