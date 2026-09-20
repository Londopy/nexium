# Security policy

## Reporting a vulnerability

Please do not open a public issue for a security problem. Use GitHub's
private vulnerability reporting on this repository ("Security" tab, "Report a
vulnerability"), or contact the maintainer (Londopy) through GitHub.

Include what you can: the compiler version (`nx version`), the platform, a
minimal `.nx` program that triggers the problem, and what you expected. You
should hear back within a week.

## What counts

Nexium promises no undefined behavior in safe code (spec section 12) and that a
panic never crosses an export boundary (S3). Reports that safe Nexium code can
be made to:

- read or write out of bounds,
- use memory after it was released, or double free,
- abort a host process that called an exported function,
- execute arbitrary code during compilation through `comptime` or
  `@embedFile` beyond the declared build inputs,

are security issues. Compiler crashes on invalid input are bugs, not
vulnerabilities, but are welcome as ordinary issues; CI fuzzes the front end
with mutated sources and the binary pattern engine with random bytes on
every push (`tests/fuzz.nx`, a fresh seed each run) and keeps any finding
as an artifact.

## Supply chain

Packages cannot run build scripts (spec 17.2); compile-time evaluation is
restricted to pure computation and declared build inputs. The compiler has no
dependencies: a C compiler is all it needs, and it is built from a C seed
that its own sources regenerate.

## Supported versions

The latest minor version receives fixes; see `docs/stability.md` for what
a version number promises.
