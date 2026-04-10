FROM centos:7

# Fix EOL CentOS 7 mirrors
RUN sed -i 's/mirrorlist/#mirrorlist/g' /etc/yum.repos.d/CentOS-* && \
    sed -i 's|#baseurl=http://mirror.centos.org|baseurl=http://vault.centos.org|g' /etc/yum.repos.d/CentOS-*

# Install build dependencies
RUN yum install -y \
        gcc \
        bzip2-devel \
        libffi-devel \
        zlib-devel \
        make \
        perl \
        wget && \
    yum clean all

# Build OpenSSL 1.1.1 from source (CentOS 7 ships 1.0.2, Python 3.10 requires 1.1.1+)
RUN wget https://www.openssl.org/source/openssl-1.1.1w.tar.gz && \
    tar -xzf openssl-1.1.1w.tar.gz && \
    cd openssl-1.1.1w && \
    ./config --prefix=/usr/local/openssl --openssldir=/usr/local/openssl --libdir=lib shared zlib && \
    make -j$(nproc) && \
    make install && \
    cd .. && \
    rm -rf openssl-1.1.1w openssl-1.1.1w.tar.gz && \
    printf '%s\n%s\n' /usr/local/openssl/lib /usr/local/openssl/lib64 > /etc/ld.so.conf.d/openssl.conf && \
    ldconfig

# Build Python 3.10 from source
RUN wget https://www.python.org/ftp/python/3.10.14/Python-3.10.14.tgz && \
    tar -xzf Python-3.10.14.tgz && \
    cd Python-3.10.14 && \
    ./configure --enable-optimizations --with-openssl=/usr/local/openssl --with-openssl-rpath=auto && \
    make altinstall && \
    cd .. && \
    rm -rf Python-3.10.14 Python-3.10.14.tgz && \
    ln -sf /usr/local/bin/python3.10 /usr/bin/python3 && \
    ln -sf /usr/local/bin/pip3.10 /usr/bin/pip && \
    python3.10 -c "import ssl; print(ssl.OPENSSL_VERSION)" >/dev/null
