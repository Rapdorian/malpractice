FROM fedora

# Setup base
RUN dnf install -y gcc wayland-devel jq git procps just
RUN dnf install -y libX11-devel libXcursor-devel  libxkbcommon-devel libXi-devel libxkbcommon-x11-devel libxcb-devel
RUN dnf install -y mesa-dri-drivers mesa-filesystem mesa-libEGL mesa-libGL mesa-libgbm mesa-libglapi mesa-va-drivers mesa-vulkan-drivers

# Setup Rust
RUN dnf install -y rustup
RUN rustup-init -y --default-toolchain 1.80.1
ENV PATH="$PATH:/root/.cargo/bin"

WORKDIR /workspaces/malpractice

# Cache fetch
RUN ln -s /workspaces/malpractice/target/fetch/git /root/.cargo/git
RUN ln -s /workspaces/malpractice/target/fetch/registry /root/.cargo/registry