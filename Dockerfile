# UnEgg - Alpine Linux static build
FROM alpine:3.20 AS builder

RUN apk add --no-cache g++ make

WORKDIR /build
COPY . .

RUN make clean && make -j$(nproc) LDFLAGS="-static"

# Final image - scratch for minimal container
FROM scratch
COPY --from=builder /build/unegg /unegg
ENTRYPOINT ["/unegg"]
