FROM rust:alpine as build

RUN apk add --no-cache opus-dev ffmpeg musl-dev

WORKDIR /app

COPY . .

RUN cargo build --release

# our final base
FROM rust:alpine

# copy the build artifact from the build stage
COPY --from=build /app/target/release/pascal .

CMD ["./pascal"]