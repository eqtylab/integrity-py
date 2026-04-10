FROM registry.access.redhat.com/ubi9/ubi:latest

RUN dnf install -y python3.11 python3-pip && \
    dnf clean all && \
    ln -sf /usr/bin/python3.11 /usr/bin/python3 && \
    ln -sf /usr/bin/pip3.11 /usr/bin/pip
