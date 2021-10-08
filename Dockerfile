FROM rust:latest

RUN apt update -y
RUN apt install -y libopus-dev ffmpeg

WORKDIR /app

COPY . .

RUN cargo build --release

CMD ["./target/release/pascal"]