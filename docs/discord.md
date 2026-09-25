# Discord Rich Presence from Nexium

[statusmith](https://github.com/Londopy/statusmith) is a tray app for
custom Discord Rich Presence, and its `nexium/` directory is the same
protocol written in Nexium, with no dependency on the app: open the local
pipe, handshake, `set_activity`, `clear`, `close`, in about three hundred
lines. It is the first package outside this repository, and the shape a
package beside an app takes (`dir`, in [packages](packages.html)).

```sh
nx add discord_rpc --git https://github.com/Londopy/statusmith --tag sdk-v0.1.0 --dir nexium
```

## A card on your profile

```nexium
import discord_rpc

fn main() -> !void {
    var a = discord_rpc.activity()
    a.details = String.from("writing the compiler in itself")
    a.state = String.from("1.3.2")
    a.start_ms = time.now()                                      // an elapsed timer
    var client = try discord_rpc.connect("1234567890123456789")  // your Application ID
    let headline = try client.set_activity(&a)                   // "Playing <headline>"
    println("on your profile: {}", .{headline})
    ...
    try client.clear()
    client.close()                                               // Discord clears the card
}
```

`activity()` is a card with nothing on it; the fields are `details` and
`state` (the two lines), `kind` (playing, listening, watching, competing),
`large_image`, `large_text`, `small_image`, `small_text` (an Art Assets key
or an `https://` link), `start_ms` and `end_ms` (an elapsed timer or a
countdown), and two buttons with a label and a URL. Empty strings and zero
timestamps are left out of the request. `connect(app_id)` opens
`\\.\pipe\discord-ipc-N` and handshakes; `set_activity(&a)` returns the
application's name; the presence stays up while the connection is open.

## What to know

- The Application ID comes from the Discord Developer Portal; no token is
  involved anywhere, and nothing but the local pipe is spoken to.
- Windows only for now: the pipe path is the one platform-specific part
  (the Unix socket variant is a small change in `open_pipe`).
- The pipe is driven through the C runtime's unbuffered `_open`, `_read`
  and `_write` via `@cImport("io.h")`, because a `FILE*` cannot switch from
  reading to writing without a seek, and pipes cannot seek.
- `connect` fails with `error.IoError` when Discord is not running, and
  with a message from Discord when the Application ID is not accepted; the
  Topo's example prints what the card would have said instead.
- `nexium/examples/presence.nx` in statusmith is a command-line front end:
  `nx run presence.nx -- --app ID --details "the build" --elapsed --hold 600`.

## A bot

For a bot, the second package outside this repository is
[nexium-discord](https://github.com/Londopy/nexium-discord): the gateway
(identify, heartbeats, resume, reconnects), the REST calls a bot makes
(messages, reactions, slash commands and their answers, and any other
call with Discord's rate limits waited out), and the events as an enum to
`match` on. It speaks through `std.websocket` and `std.http`, with TLS
from [nxtls](https://github.com/Londopy/nxtls) in the TLS slot, so it
needs Nexium 1.4 and, until nxtls moves to `random.secure`, Linux, macOS
or a BSD.

```toml
[dependencies]
discord = { git = "https://github.com/Londopy/nexium-discord", tag = "v0.1.0" }
```

Its [pingbot](https://github.com/Londopy/nexium-discord/tree/main/examples/pingbot)
answers `/ping` and `!ping`; the token comes from `$DISCORD_TOKEN`.
