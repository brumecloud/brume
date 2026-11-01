# syntax=docker/dockerfile:1

# nginx 1.30 (stable), built to run as UID 101: no root, no capability needed on port 8080.
FROM nginxinc/nginx-unprivileged:1.30-alpine-slim@sha256:5d8bb9437d74a600ec5dbf90c035451c724dc33eac8d3a71078f6f8612c345f6
# Owned by root and read-only for nginx. The image's entrypoint scripts and default site are
# left out: nginx starts directly on this config.
COPY nginx.conf /etc/brume-demo/nginx.conf
COPY index.html /srv/www/index.html
USER 101
EXPOSE 8080
ENTRYPOINT ["nginx", "-c", "/etc/brume-demo/nginx.conf"]
CMD []
