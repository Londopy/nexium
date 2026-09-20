# A network service

`std.net` gives you sockets; `std.http` gives you a client and a server
over them. This program is both: a service with three routes, started on a
thread, and a client that calls each route and prints what came back. With a
port on the command line it serves until you stop it.

{{include topo/code/service.nx}}

{{output topo/code/service.expected}}

```bash
$ nx run topo/code/service.nx -- 8080
serving on http://127.0.0.1:8080/greet?name=you
```

## Handlers and routes

```nexium
fn greet(req: *http.Request) -> http.Response {
    var body = String.from("hello, ")
    body.append(req.param("name") orelse "stranger")
    return http.text(200, body)
}
```

A handler is a plain function from a request to a response. `req.param`
reads a query parameter, `req.header` a header, `req.body` the body;
`http.text`, `http.json`, `http.html` and `http.respond(status,
content_type, body)` build responses, `http.not_found()` and
`http.redirect(url)` the two common special ones. A `Router` maps a method
and a path to a handler, and `serve_static(dir)` serves files for the paths
no route claims.

Because handlers are function values, the router holds a
`fn(*Request) -> Response` per route, and everything a handler can do is
visible in its effects. A handler that allocates is normal; one that has
`shared_mutable` is a handler that touches state other requests see, and
`nx effects` will show it.

## The server

```nexium
    var server = http.Server.bind("127.0.0.1", s.port) catch { return }
    var r = routes()
    for _ in 0..s.n { server.serve_one(&r, 5000) catch { break } }
```

`Server.bind` listens; `serve_one(&router, timeout_ms)` handles one
connection and returns, `serve(&router)` loops forever. The server handles
one request at a time, which is what a tool, a local dashboard or a test
needs. For the self-test it runs on a thread from chapter 16, answers three
requests and stops, so the program can also be the client.

## The client

```nexium
    let h = try http.get(format("{}/health", .{base})[..])
    println("{} {}", .{h.status, h.body})
```

`http.get(url)`, `http.post(url, content_type, body)` and the general
`http.request(method, url, headers, body)` speak HTTP/1.1, read bodies by
`Content-Length` or chunked encoding, and follow redirects. Plain `http://`
only: TLS would need a C library, and `@cImport` (chapter 20) is how one is
brought in.

Underneath, `std.net.TcpStream.connect(host, port)`, `send`, `recv`,
`recv_all`, `reader()` and `writer()` (buffered, from `std.stream`) are what
the HTTP layer is written with, and `TcpListener.bind` / `accept` on the
other side. The program uses one of these directly: binding a listener on
port 0 to ask the operating system for a free port, then closing it and
giving the number to the server thread.

## Errors and effects

Every call that touches the network `blocks` and can fail with
`ConnectionRefused`, `Timeout`, `NotFound` (a name that does not resolve) or
`IoError`. Timeouts are per socket, in milliseconds, and expire with
`error.Timeout`. `try` and `catch` handle them like any other error; a
service that must not go down catches at the top of its loop and keeps
serving.

Next: [your first GUI](18-your-first-gui.html).
