# The Nexium compiler with the Zig toolchain, on Debian.
#
#     docker run --rm -v "$PWD":/work ghcr.io/londopy/nexium run hello.nx
#     docker run --rm -it --entrypoint sh ghcr.io/londopy/nexium
#
# nx is built from the compiler's one C file, bootstrap/nx.c, by the image's
# own gcc, so the same Dockerfile serves amd64 and arm64. Zig sits at
# /usr/local/zig, where nx looks for a bundled toolchain (next to its own
# directory), so nx doctor reports it as bundled and nothing needs the PATH.
# Built and published by .github/workflows/docker.yml at each release.

ARG ZIG_VERSION=0.14.1

FROM debian:bookworm-slim@sha256:3783cc01769c7b2b1b83a5c5ad96c815348e28ed7da68e2e3687004faa906251 AS build
ARG ZIG_VERSION
ARG TARGETARCH
RUN apt-get update && apt-get install -y --no-install-recommends gcc libc6-dev curl xz-utils ca-certificates python3 && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY bootstrap/nx.c bootstrap/nx.c
RUN case "$(uname -m)" in x86_64) march="-march=x86-64" ;; *) march="" ;; esac \
 && gcc -std=gnu11 -O2 -w -fno-strict-aliasing $march -o nx bootstrap/nx.c -lm -lc -lpthread
# Zig for this architecture, verified against ziglang.org's index
RUN case "$TARGETARCH" in amd64) zarch=x86_64 ;; arm64) zarch=aarch64 ;; *) echo "no zig for $TARGETARCH" && exit 1 ;; esac \
 && name="zig-$zarch-linux-$ZIG_VERSION" \
 && curl -fsSL "https://ziglang.org/download/$ZIG_VERSION/$name.tar.xz" -o zig.tar.xz \
 && curl -fsSL https://ziglang.org/download/index.json -o index.json \
 && want="$(python3 -c "import json; print(json.load(open('index.json'))['$ZIG_VERSION']['$zarch-linux']['shasum'])")" \
 && echo "$want  zig.tar.xz" | sha256sum -c - \
 && mkdir -p /out/zig && tar -xJf zig.tar.xz -C /out/zig --strip-components=1

FROM debian:bookworm-slim@sha256:3783cc01769c7b2b1b83a5c5ad96c815348e28ed7da68e2e3687004faa906251
LABEL org.opencontainers.image.source="https://github.com/Londopy/nexium" \
      org.opencontainers.image.description="The Nexium language: the compiler with its Zig toolchain, the standard library, the examples and the docs" \
      org.opencontainers.image.licenses="MIT"
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl git && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/nx /usr/local/bin/nx
COPY --from=build /out/zig /usr/local/zig
COPY examples /usr/local/share/nexium/examples
COPY std /usr/local/share/nexium/std
COPY docs /usr/local/share/nexium/docs
COPY README.md LICENSE CHANGELOG.md /usr/local/share/nexium/
WORKDIR /work
ENTRYPOINT ["nx"]
CMD ["doctor"]
