FROM ubuntu:24.04

SHELL ["/bin/bash", "-lc"]

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update -y && \
    apt-get install -y --no-install-recommends \
      ca-certificates \
      curl \
      git \
      unzip \
      zip \
      xz-utils \
      jq \
      python3 \
      python3-pip \
      build-essential \
      golang-go \
      elixir \
      openjdk-21-jdk-headless \
      kotlin \
      docker.io \
      libgmp-dev \
      php-cli && \
    rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://dot.net/v1/dotnet-install.sh | bash /dev/stdin --channel 9.0 --install-dir /root/.dotnet
RUN curl -fsSL https://sh.rustup.rs | bash -s -- -y --profile minimal

ARG SWIFT_VERSION=6.2.4
ARG SWIFT_PLATFORM=ubuntu24.04
RUN apt-get update -y && \
    apt-get install -y --no-install-recommends \
      binutils \
      gnupg2 \
      libc6-dev \
      libcurl4-openssl-dev \
      libedit2 \
      libgcc-13-dev \
      libncurses-dev \
      libpython3-dev \
      libsqlite3-0 \
      libstdc++-13-dev \
      libxml2-dev \
      libz3-dev \
      pkg-config \
      tzdata \
      zlib1g-dev && \
    rm -rf /var/lib/apt/lists/*

RUN set -euo pipefail; \
    curl -fsSL "https://download.swift.org/swift-${SWIFT_VERSION}-release/ubuntu2404/swift-${SWIFT_VERSION}-RELEASE/swift-${SWIFT_VERSION}-RELEASE-${SWIFT_PLATFORM}.tar.gz" -o /tmp/swift.tar.gz; \
    mkdir -p /opt/swift; \
    tar -xzf /tmp/swift.tar.gz -C /opt/swift --strip-components=1; \
    rm -f /tmp/swift.tar.gz

ARG ZIG_VERSION=0.13.0
RUN set -euo pipefail; \
    if command -v zig >/dev/null 2>&1; then exit 0; fi; \
    curl -fsSL "https://ziglang.org/download/${ZIG_VERSION}/zig-linux-x86_64-${ZIG_VERSION}.tar.xz" -o /tmp/zig.tar.xz; \
    tar -xJf /tmp/zig.tar.xz -C /opt; \
    ln -sf "/opt/zig-linux-x86_64-${ZIG_VERSION}/zig" /usr/local/bin/zig; \
    rm -f /tmp/zig.tar.xz

RUN set -euo pipefail; \
    (curl -fsSL --connect-timeout 30 https://deno.land/install.sh | DENO_INSTALL=/opt/deno sh \
     && ln -sf /opt/deno/bin/deno /usr/local/bin/deno) \
    || echo "WARNING: Deno install failed (network?), TypeScript benchmarks will be skipped"

RUN if [ -x /opt/swift/usr/bin/swift ]; then echo "Swift OK"; \
    elif [ -x /opt/swift/bin/swift ]; then ln -sf /opt/swift/bin /opt/swift/usr/bin; echo "Swift symlinked"; \
    else echo "WARNING: Swift binary not found"; fi

ENV PATH="/opt/swift/usr/bin:/root/.dotnet:/root/.cargo/bin:${PATH}"
RUN cargo install hyperfine --version 1.19.0

WORKDIR /workspace/vibelang
COPY benchmarks/third_party/docker/entrypoint.sh /usr/local/bin/vibe-third-party-bench
RUN chmod +x /usr/local/bin/vibe-third-party-bench

ENTRYPOINT ["/usr/local/bin/vibe-third-party-bench"]
