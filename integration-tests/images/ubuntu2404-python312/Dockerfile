FROM ubuntu:24.04

RUN apt-get update && \
    apt-get install -y python3.12 python3-pip && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* && \
    ln -sf /usr/bin/python3.12 /usr/bin/python3 && \
    ln -sf /usr/bin/pip3 /usr/bin/pip
