FROM rust:latest as build

RUN apt update && apt install -y libopus-dev ffmpeg

WORKDIR /app

COPY . .

RUN cargo build --release

# our final base
FROM ubuntu:bionic

RUN apt update && apt install -y libopus-dev ffmpeg youtube-dl

# copy the build artifact from the build stage
COPY --from=build /app/target/release/pascal /pascal

CMD ["./pascal"]