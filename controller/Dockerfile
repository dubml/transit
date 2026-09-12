FROM golang:1.26-bookworm AS build
WORKDIR /src
COPY crates/xds/proto crates/xds/proto
COPY controller controller
WORKDIR /src/controller
RUN CGO_ENABLED=0 GOWORK=off go build -trimpath -o /transit-controller ./cmd/transit-controller

FROM gcr.io/distroless/static-debian12:nonroot
COPY --from=build /transit-controller /transit-controller
USER 65532:65532
ENTRYPOINT ["/transit-controller"]
