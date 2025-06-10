FROM alpine

WORKDIR /app

COPY target/debug/webhook-puller ./webhook-puller

ENTRYPOINT ["./webhook-puller"]
