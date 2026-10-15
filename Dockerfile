# syntax=docker/dockerfile:1
ARG RUST_VERSION=1.99.0
FROM rust:${RUST_VERSION}-slim-bookworm AS build
WORKDIR /src
RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential cmake perl pkg-config ca-certificates git python3 \
    && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY assets ./assets
COPY scripts/collect_notices.py ./scripts/collect_notices.py
COPY packaging/notices ./packaging/notices
RUN cargo build --locked --release \
    && python3 scripts/collect_notices.py --output /notices

FROM debian:bookworm-slim AS runtime
ARG VERSION=0.4.0
ARG VCS_REF=unknown
LABEL org.opencontainers.image.title="XMQR Admin UI" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.revision="${VCS_REF}" \
      org.opencontainers.image.source="https://github.com/licodevone/xmqr-admin" \
      org.opencontainers.image.licenses="MIT"
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libgcc-s1 \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 xmqr-admin \
    && useradd --uid 10001 --gid 10001 --no-create-home --home-dir /nonexistent --shell /usr/sbin/nologin xmqr-admin
COPY LICENSE ORIGIN.json /usr/share/licenses/xmqr-admin/
COPY docs/third-party-licenses.md /usr/share/licenses/xmqr-admin/third-party-licenses.md
COPY --from=build /notices /usr/share/licenses/xmqr-admin/third-party-notices
COPY --from=build /src/target/release/xmqr-admin /usr/local/bin/xmqr-admin
RUN test "$(/usr/local/bin/xmqr-admin --version)" = "xmqr-admin ${VERSION}"
ENV XMQR_ADMIN_BIND=127.0.0.1:9080 \
    XMQR_ADMIN_MONITOR_BIND=127.0.0.1:9090
USER 10001:10001
STOPSIGNAL SIGTERM
ENTRYPOINT ["/usr/local/bin/xmqr-admin"]
