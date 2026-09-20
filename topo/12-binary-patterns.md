# Binary patterns

Most languages parse binary formats with a pile of shifts, masks and
offsets that is wrong in one place. Nexium borrows an idea from Erlang: a
pattern that *describes the bytes*, which the compiler turns into the
shifts and masks, with every length checked.

{{include topo/code/binary.nx}}

{{output topo/code/binary.expected}}

## Reading

```nexium
    match data {
        <<0x89, 'P', 'N', 'G', 0x0d, 0x0a, 0x1a, 0x0a,
          len:32/big, 'I', 'H', 'D', 'R',
          width:32/big, height:32/big, depth:8, color:8, compression:8, filter:8, interlace:8,
          crc:32/big, rest:bytes>> => { ... }
```

A binary pattern is a list of *segments* between `<<` and `>>`, matched
against a `[]u8` from left to right. A segment is either a literal that
must be there (`0x89`, `'P'`) or a binding with a size in bits:

- `width:32/big` binds a 32-bit big-endian unsigned integer. The size
  decides the type: up to 64 bits is an integer of the smallest fitting
  width (`u8`, `u16`, `u32`, `u64`), so `width` is a `u32` and `depth:8` a
  `u8`, with no cast.
- `version:4, ihl:4` split one byte into two four-bit fields; sizes need
  not be multiples of eight.
- `payload:len*8` has a size computed from an earlier binding: length-
  prefixed data in one line.
- `rest:bytes` takes everything that is left, as a `[]u8` view of the
  input. It is how a pattern says "and more follows"; without it the
  pattern must consume the input exactly.

The modifiers after `/` are `big` (the default), `little`, `native`,
`signed`, `unsigned`, `float` and `utf8`. When the input is too short for
the pattern, the arm simply does not match and the next one is tried, which
is why `parse_png` can say `Truncated` for something that has the signature
and nothing after it: the second arm needs only the four bytes.

Nothing is copied by a pattern. The integer bindings are read out of the
input; `payload` and `rest` are views into it.

## Writing

```nexium
    let written = <<3:16/little, "abc", 300:16/big, 1:1, 0:3, 5:4>> into buf[..] catch { return }
```

The same syntax constructs bytes. `into` names a `[]mut u8` buffer; the
result is the prefix that was written, or `error.BufferTooSmall`. Values
larger than their size are an error at compile time when they are literals,
and checked when they are not.

## Where it pays

Network protocols, file headers, embedded devices, anything with a wire
format. The frames example is the shape of every length-prefixed protocol:
match a length and that many bytes, keep the tail, repeat. `std.bytes` sits
beside patterns with `hex`, `base64`, `crc32` and the endian read/write
helpers for the cases where a pattern is more than you need.

Next: [compile time](13-comptime.html).
