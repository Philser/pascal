FROM rust:latest as build

RUN apt update && apt install -y libopus-dev ffmpeg

WORKDIR /app

COPY . .

RUN cargo build --release

# our final base
FROM ubuntu:20.04

ARG DEBIAN_FRONTEND=noninteractive

RUN apt update && apt install -y libopus-dev ffmpeg pip && pip install --upgrade youtube-dl

# copy the build artifact from the build stage
COPY --from=build /app/target/release/pascal /pascal

CMD ["./pascal"]