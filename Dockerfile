FROM rust:alpine

# RUN apk update -y
RUN apk add --no-cache opus-dev ffmpeg musl-dev

WORKDIR /app

COPY . .

RUN cargo build --release

RUN cp ./target/release/pascal ./

RUN rm -rf ./target

CMD ["./pascal"]