FROM registry.access.redhat.com/ubi10/ubi:latest

RUN dnf install -y python3.12 python3-pip && \
    dnf clean all && \
    ln -sf /usr/bin/python3.12 /usr/bin/python3 && \
    ln -sf /usr/bin/pip3 /usr/bin/pip
