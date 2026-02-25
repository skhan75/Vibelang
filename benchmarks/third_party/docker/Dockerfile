FROM ubuntu:24.04

SHELL ["/bin/bash", "-lc"]

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update -y && \
    apt-get install -y --no-install-recommends \
      ca-certificates \
      curl \
      git \
      unzip \
      jq \
      python3 \
      python3-pip \
      build-essential \
      docker.io && \
    rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://dot.net/v1/dotnet-install.sh | bash /dev/stdin --channel 9.0 --install-dir /root/.dotnet
RUN curl -fsSL https://sh.rustup.rs | bash -s -- -y --profile minimal

ENV PATH="/root/.dotnet:/root/.cargo/bin:${PATH}"
RUN cargo install hyperfine --version 1.19.0

WORKDIR /workspace/VibeStack/vibelang
COPY vibelang/benchmarks/third_party/docker/entrypoint.sh /usr/local/bin/vibe-third-party-bench
RUN chmod +x /usr/local/bin/vibe-third-party-bench

ENTRYPOINT ["/usr/local/bin/vibe-third-party-bench"]
