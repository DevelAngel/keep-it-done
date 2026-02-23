FROM scratch AS app

COPY --chmod=755 kid-server /bin/kid-server
COPY --chmod=755 site /site

ENV LEPTOS_SITE_ROOT="/site"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"

ENTRYPOINT ["/bin/kid-server", "--listen", "0.0.0.0:8080"]

EXPOSE 3000
EXPOSE 8080

USER 1001:1001
COPY --chown=1001:1001 --chmod=750 data /data/tasks
VOLUME ["/data/tasks"]
WORKDIR /data
