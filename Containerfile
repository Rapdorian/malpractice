FROM fedora

# Setup base
RUN dnf install -y gcc
RUN dnf install -y wayland-devel
RUN dnf install -y jq

# Setup Rust
RUN dnf install -y rustup
RUN rustup-init -y --default-toolchain 1.80.1
ENV PATH="$PATH:/root/.cargo/bin"

VOLUME /project
WORKDIR /project

# Install devcontainer deps
RUN dnf install -y procps
RUN dnf install -y just

# Cache fetch
RUN ln -s /project/target/fetch/git /root/.cargo/git
RUN ln -s /project/target/fetch/registry /root/.cargo/registry
