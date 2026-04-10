FROM alpine:3.22

RUN apk add --no-cache python3 py3-pip && \
    ln -sf /usr/bin/pip3 /usr/bin/pip && \
    python3 --version && \
    pip --version
