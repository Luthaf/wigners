FROM quay.io/pypa/manylinux2010_x86_64

# We need a more recent audiwheel
RUN /opt/_internal/pipx/venvs/auditwheel/bin/python -m pip install --upgrade auditwheel

# Manually download rustup-init version 1.25.1, since 1.25.2 requires a more
# modern glibc. We can remove this if https://github.com/rust-lang/rustup/issues/3203
# is solved by making rustup-init.sh able to detect the glibc version.
RUN curl -O https://static.rust-lang.org/rustup/archive/1.25.1/x86_64-unknown-linux-gnu/rustup-init && \
    chmod +x ./rustup-init && \
    ./rustup-init -y --profile minimal --default-toolchain 1.63

ENV PATH="/root/.cargo/bin:${PATH}"
