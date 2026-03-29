# Deployment Instructions

Let's try to create a statically-linked (not dynamically linked) executable
so that we can build a container image
[from scratch](https://www.geeksforgeeks.org/devops/docker-scratch-based-images/).
That means that the container has no other executables.

## Container Image for kid-server

After cross-compiling `kid-server` for the target platform,
copy it to our deployment directory.
Copy the `site` directory as well.
Furthermore, create an empty `data` directory because the container image
from scratch does not have any `mkdir` command which we can use.

### x86_64 with musl libc

```console
$ mkdir -p deployment/data
$ cp -av -t deployment \
  target/x86_64-unknown-linux-musl/release/kid-server \
  target/site

$ pushd deployment
$ podman build . --tag localhost/kid-server:test --arch amd64
$ popd

$ podman run --rm --name kid-server --publish 3000:3000 --publish 8080:8080 localhost/kid-server:test -vvvv
```

### aarch64 with musl libc for Rasperry Pi 3+

```console
$ mkdir -p deployment/data
$ cp -av -t deployment \
  target/aarch64-unknown-linux-musl/release/kid-server \
  target/site

$ podman build . --tag localhost/kid-server:test --arch arm64
```

## Podman-Quadlet Services

### Build Unit

Instead of creating the container on the host which compiles the software,
we can create the container on the target platform with systemd units
using Podman Quadlet.

`/etc/containers/systemd/kid.build`:

```systemd
[Unit]
Description=Keep It Done - Image Generator
AssertPathExists=/etc/containers/build/kid

[Build]
SetWorkingDirectory=/etc/containers/build/kid
File=Dockerfile
ImageTag=localhost/kid-server:latest
```

The provided `Dockerfile` requires the following file structure:

```console
$ tree /etc/containers/build/kid
[drwxr-x---]  /etc/containers/build/kid
├── [drwxr-x---]  data
├── [-rw-r-----]  Dockerfile
├── [-rw-r-----]  kid-server
└── [drwxr-xr-x]  site
    ├── [-rw-r-----]  favicon-16x16.png
    ├── [-rw-r-----]  favicon-32x32.png
    ├── [-rw-r-----]  favicon.ico
    └── [drwxr-xr-x]  pkg
        ├── [-rw-r-----]  kid_bg.wasm.d.ts
        ├── [-rw-r-----]  kid.css
        ├── [-rw-r-----]  kid.d.ts
        ├── [-rw-r-----]  kid.js
        └── [-rw-r-----]  kid.wasm
```

Create the container by running the service:

```console
$ systemctl daemon-reload
$ systemctl start kid-build.service
# Container localhost/kid-server:latest is now created
```

### Container Unit

Create user and group for kid service:

```console
$ groupadd --gid 2333 kid
$ useradd --uid 2333 --gid 2333 --no-create-home --shell /bin/false kid
```

To create the systemd service, we can also use Podman-Quadlet.

`/etc/containers/systemd/kid.container`:

```systemd
[Unit]
Description=Keep It Done
Wants=network-online.target
After=network-online.target
AssertPathIsDirectory=/var/lib/kid/data
RequiresMountsFor=/var/lib/kid/data

[Container]
ContainerName=kid
Image=localhost/kid:latest
Pull=never
ReadOnly=true
NoNewPrivileges=true
Volume=/var/lib/kid/data:/data/tasks:rw,z
User=1001
UIDMap=0:10000:999
UIDMap=1001:2333:1
Group=1001
GIDMap=0:10000:999
GIDMap=1001:2333:1
ExposeHostPort=3000
ExposeHostPort=8080
Exec=-vv
Label=traefik.enable=True
Label=traefik.http.routers.kid.rule=Host(`kid.example.com`)
Label=traefik.http.routers.kid.tls=True

[Service]
TimeoutStartSec=900
Restart=always
RestartSec=1min
RestartSteps=10
RestartMaxDelaySec=4h

[Install]
WantedBy=traefik.service
```

Start the service and check the logs:

```console
$ systemctl daemon-reload
$ systemctl start kid.service
$ journalctl -I --follow --unit kid.service
```
