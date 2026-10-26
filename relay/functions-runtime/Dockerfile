FROM denoland/deno:2.3.3
WORKDIR /runtime
COPY deno.json main.ts runtime.ts worker.ts ./
# Everything from here runs as the unprivileged deno user, who owns the module cache it fills.
USER deno
# The SDK every build imports, fetched once at build time.
RUN deno cache main.ts worker.ts
ENV RELAY_FUNCTIONS_DATA=/tmp/relay-functions PORT=8000
EXPOSE 8000
CMD ["deno", "run", "--unstable-worker-options", "--allow-net", "--allow-env", "--allow-read", "--allow-write=/tmp/relay-functions", "--allow-run=/usr/bin/deno", "--allow-import", "main.ts"]
