FROM rust:1-bullseye

RUN mkdir /root/timecat
COPY src /root/timecat/src
COPY Cargo.toml /root/timecat
COPY build.rs /root/timecat

WORKDIR /root/timecat

ENV RUSTFLAGS="-C target-cpu=native"
RUN cargo build --release

RUN mv /root/timecat/target/release/timecat /timecat
RUN rm -rf /root

CMD [ "/timecat", "--no-color", "--uci" ]