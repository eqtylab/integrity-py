FROM registry.access.redhat.com/ubi9/ubi:latest

RUN dnf install -y python3.11 python3.11-pip && \
    dnf clean all && \
    ln -sf /usr/bin/python3.11 /usr/bin/python3 && \
    ln -sf /usr/bin/pip3.11 /usr/bin/pip && \
    python3.11 -m pip --version
