FROM rust:1-buster

RUN rustup set profile default
RUN rustup install stable beta nightly
RUN rustup default stable
RUN rustup component add --toolchain stable rust-src rls
RUN rustup component add --toolchain beta rust-src rls
RUN rustup component add --toolchain nightly rust-src rls
RUN cargo install cargo-edit
RUN cargo install basic-http-server
ADD https://github.com/rust-analyzer/rust-analyzer/releases/download/nightly/rust-analyzer-linux /usr/local/bin/rust-analyzer
RUN chmod +x /usr/local/bin/rust-analyzer
RUN mkdir -p /workspace/.cargo/bin
RUN ln -s /usr/local/cargo/bin/* /usr/local/bin
