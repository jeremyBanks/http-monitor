FROM rust:1-buster

RUN rustup set profile default
RUN rustup install stable beta nightly
RUN rustup default stable
RUN rustup component add --toolchain stable rust-src rls
RUN rustup component add --toolchain beta rust-src rls
RUN rustup component add --toolchain nightly rust-src rls rust-analyzer-preview
RUN cargo install cargo-edit
RUN cargo install https
RUN cargo clean
