# scsock

Simple states managment app that using sockets.
Requires UNIX, and may be Linux.

## Example usage

Here is an example of configuration file:

```toml
socket = "/tmp/my.socket"
# optional field, default: false
remove-socket-if-exists = true

# First action is default state.
[actions]
foo = "echo foo"
bar = { name = "Bar", do = "echo bar" }
```

1. Build executable (`target/release/scsock`) by running `cargo b -r`.
2. Run the server: `scsock -c path/to/config.toml start`

```console
$ scsock -c path/to/config.toml next
New state: Bar
$ scsock -c path/to/config.toml set 0
New state: foo
$ scsock -c path/to/config.toml get
New state: foo

# This commands does not requires running server:
$ scsock -c path/to/config.toml list
0: foo (echo foo)
1: Bar (echo bar)
```

## Protocol

Here is a table of messages, that can be sent or received:

| Name        | Bytes             | Description                                         |
|-------------|-------------------|-----------------------------------------------------|
| `GetStatus` | `[0]`             | Gets the current status. Returns `ReStatus`         |
| `SetStatus` | `[1, ID]`         | Sets state to ID. Returns `ReStatus` on success     |
| `NextID`    | `[2]`             | Go to next status ID. Returns `ReStatus` on success |
| ...         | ...               | ...                                                 |
| `ReStatus`  | `[128, len, ...]` | **Reply**: status. String is not zero-terminated    |
| `ReErrNoID` | `[129]`           | **Reply**: error, id doesn't exists                 |
| `ReErrIdiot`| `[130]`           | **Reply**: error, client sent server's response     |
| `ReErrUnkwn`| `[131]`           | **Reply**: error, unknown message format            |

Other kinds reserved for future.

