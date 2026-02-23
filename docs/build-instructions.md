# Build Instructions for Linux

## Requirements

```console
$ cargo binstall cargo-leptos
$ cargo binstall cargo-generate
$ rustup target add wasm32-unknown-unknown
```

### For cross-compilation

```console
$ cargo binstall cross
$ cargo binstall ripgrep
```

and:

- `podman` or `docker`

## x86_64 (native compilation)

### kid-cli

```console
$ cargo build -p kid-cli --release --locked
   Compiling ...
    Finished `release` profile [optimized] target(s) in 56.47s

$ file target/release/kid
target/release/kid: ELF 64-bit LSB pie executable, x86-64, version 1 (SYSV), dynamically linked, interpreter /lib64/ld-linux-x86-64.so.2, for GNU/Linux 3.2.0, BuildID[sha1]=08b493c9c8a764b1942443594eccd94c857b2e14, not stripped

$ ldd target/release/kid
        linux-vdso.so.1 (0x00007f73ad9de000)
        libgcc_s.so.1 => /usr/lib/x86_64-linux-gnu/libgcc_s.so.1 (0x00007f73ad836000)
        libm.so.6 => /usr/lib/x86_64-linux-gnu/libm.so.6 (0x00007f73ad740000)
        libc.so.6 => /usr/lib/x86_64-linux-gnu/libc.so.6 (0x00007f73ad54a000)
        /lib64/ld-linux-x86-64.so.2 (0x00007f73ad9e0000)
```

### kid-server incl. kid-frontend
 
```console
$ cargo leptos build --release --bin-cargo-args="--locked" --lib-cargo-args="--locked"

$ file target/release/kid-server
target/release/kid-server: ELF 64-bit LSB pie executable, x86-64, version 1 (SYSV), dynamically linked, interpreter /lib64/ld-linux-x86-64.so.2, for GNU/Linux 3.2.0, BuildID[sha1]=e2b921aff4669b57dcfca8d089a37c892ba89092, not stripped

$ ldd target/release/kid-server
        linux-vdso.so.1 (0x00007f7bab36b000)
        libgcc_s.so.1 => /usr/lib/x86_64-linux-gnu/libgcc_s.so.1 (0x00007f7bab32d000)
        libm.so.6 => /usr/lib/x86_64-linux-gnu/libm.so.6 (0x00007f7bab237000)
        libc.so.6 => /usr/lib/x86_64-linux-gnu/libc.so.6 (0x00007f7baac0a000)
        /lib64/ld-linux-x86-64.so.2 (0x00007f7bab36d000)
```

## x86_64 (cross compilation using glibc)

### Build kid-cli

```console
$ cross build --target x86_64-unknown-linux-gnu -p kid-cli --release --locked
info: syncing channel updates for 'stable-x86_64-unknown-linux-gnu'

  stable-x86_64-unknown-linux-gnu unchanged - rustc 1.93.1 (01f6ddf75 2026-02-11)

info: checking for self-update
   Compiling ...
    Finished `release` profile [optimized] target(s) in 1m 17s

$ file target/x86_64-unknown-linux-gnu/release/kid
target/x86_64-unknown-linux-gnu/release/kid: ELF 64-bit LSB pie executable, x86-64, version 1 (SYSV), dynamically linked, interpreter /lib64/ld-linux-x86-64.so.2, for GNU/Linux 2.6.32, BuildID[sha1]=d395b6d21a8a9a4ce25ab27885ce76f0a98242db, not stripped 

$ ldd target/x86_64-unknown-linux-gnu/release/kid
        linux-vdso.so.1 (0x00007f2b5daa1000)
        libgcc_s.so.1 => /lib/x86_64-linux-gnu/libgcc_s.so.1 (0x00007f2b5da5d000)
        librt.so.1 => /lib/x86_64-linux-gnu/librt.so.1 (0x00007f2b5da58000)
        libpthread.so.0 => /lib/x86_64-linux-gnu/libpthread.so.0 (0x00007f2b5da53000)
        libm.so.6 => /lib/x86_64-linux-gnu/libm.so.6 (0x00007f2b5d96d000)
        libdl.so.2 => /lib/x86_64-linux-gnu/libdl.so.2 (0x00007f2b5d5fb000)
        libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x00007f2b5d405000)
        /lib64/ld-linux-x86-64.so.2 (0x00007f2b5daa3000)
```

### Build kid-frontend

```console
$ cargo leptos build --frontend-only --release --lib-cargo-args="--locked"
    Metadata keys ["env", "watch"] from metadata.leptos are not recognized and will be ignored
   Compiling ...
    Finished `release` profile [optimized] target(s) in 2m 23s
       Cargo finished cargo build --package=kid-frontend --lib --target-dir=/home/develangel/workspace3/rust/keep-it-done/target/front --target=wasm32-unknown-unknown --no-default-features --locked --release
       Front generating JS/WASM with wasm-bindgen
       Using wasm-bindgen version 0.2.108 detected in project
[swc_ecma_transforms_optimization] tree-shaker; pass=0
[swc_ecma_transforms_optimization] tree-shaker; pass=1
[swc_ecma_transforms_optimization] tree-shaker; pass=2
[swc_ecma_transforms_optimization] tree-shaker; pass=0
    Finished generating JS/WASM for front in 4.757749452s
    Tailwind finished tailwindcss --input style/tailwind.css --output /home/develangel/workspace3/rust/keep-it-done/target/tmp/tailwind.css --minify
       
$ file target/site/pkg/kid.wasm
target/site/pkg/kid.wasm: WebAssembly (wasm) binary module version 0x1 (MVP)
```

### Build kid-server

```console
$ export LEPTOS_BIN_CARGO_COMMAND="cross"
$ export LEPTOS_BIN_TARGET_TRIPLE="x86_64-unknown-linux-gnu"

$ env | rg LEPTOS
LEPTOS_BIN_CARGO_COMMAND=cross
LEPTOS_BIN_TARGET_TRIPLE=x86_64-unknown-linux-gnu

$ cargo leptos build --release --bin-cargo-args="--locked" --lib-cargo-args="--locked"
    Metadata keys ["env", "watch"] from metadata.leptos are not recognized and will be ignored
    Finished `release` profile [optimized] target(s) in 0.28s
       Cargo finished cargo build --package=kid-frontend --lib --target-dir=/var/home/develangel/workspace3/rust/keep-it-done/target/front --target=wasm32-unknown-unknown --no-default-features --locked --release
       Front generating JS/WASM with wasm-bindgen
       Using wasm-bindgen version 0.2.108 detected in project
[swc_ecma_transforms_optimization] tree-shaker; pass=0
[swc_ecma_transforms_optimization] tree-shaker; pass=1
[swc_ecma_transforms_optimization] tree-shaker; pass=2
[swc_ecma_transforms_optimization] tree-shaker; pass=0
    Finished generating JS/WASM for front in 4.629138266s
    Tailwind finished tailwindcss --input style/tailwind.css --output /var/home/develangel/workspace3/rust/keep-it-done/target/tmp/tailwind.css --minify
info: syncing channel updates for 'stable-x86_64-unknown-linux-gnu'

  stable-x86_64-unknown-linux-gnu unchanged - rustc 1.93.1 (01f6ddf75 2026-02-11)

info: checking for self-update
   Compiling ...
    Finished `release` profile [optimized] target(s) in 3m 59s
       Cargo finished cross build --package=kid-server --bin=kid-server --target=x86_64-unknown-linux-gnu --no-default-features --locked --release

$ file target/x86_64-unknown-linux-gnu/release/kid-server
target/x86_64-unknown-linux-gnu/release/kid-server: ELF 64-bit LSB pie executable, x86-64, version 1 (SYSV), dynamically linked, interpreter /lib64/ld-linux-x86-64.so.2, for GNU/Linux 2.6.32, BuildID[sha1]=8da923ff354b46aed4d6532985e98b07eca240ab, not stripped

$ ldd target/x86_64-unknown-linux-gnu/release/kid-server
        linux-vdso.so.1 (0x00007fe5ae4d9000)
        libgcc_s.so.1 => /lib/x86_64-linux-gnu/libgcc_s.so.1 (0x00007fe5ae495000)
        librt.so.1 => /lib/x86_64-linux-gnu/librt.so.1 (0x00007fe5ae490000)
        libpthread.so.0 => /lib/x86_64-linux-gnu/libpthread.so.0 (0x00007fe5ae48b000)
        libm.so.6 => /lib/x86_64-linux-gnu/libm.so.6 (0x00007fe5ae3a5000)
        libdl.so.2 => /lib/x86_64-linux-gnu/libdl.so.2 (0x00007fe5ae3a0000)
        libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x00007fe5adc0a000)
        /lib64/ld-linux-x86-64.so.2 (0x00007fe5ae4db000)
```

## x86_64 (cross compilation using musl)

### Build kid-cli

```console
$ cross build --target x86_64-unknown-linux-musl -p kid-cli --release --locked
info: syncing channel updates for 'stable-x86_64-unknown-linux-gnu'

  stable-x86_64-unknown-linux-gnu unchanged - rustc 1.93.1 (01f6ddf75 2026-02-11)

info: checking for self-update
   Compiling ...
    Finished `release` profile [optimized] target(s) in 59.34s

$ file target/x86_64-unknown-linux-musl/release/kid
target/x86_64-unknown-linux-musl/release/kid: ELF 64-bit LSB pie executable, x86-64, version 1 (SYSV), static-pie linked, not stripped

$ ldd target/x86_64-unknown-linux-musl/release/kid
        statically linked
```

### Build kid-frontend

```console
$ cargo leptos build --frontend-only --release --lib-cargo-args="--locked"
    Metadata keys ["env", "watch"] from metadata.leptos are not recognized and will be ignored
   Compiling ...
    Finished `release` profile [optimized] target(s) in 2m 23s
       Cargo finished cargo build --package=kid-frontend --lib --target-dir=/home/develangel/workspace3/rust/keep-it-done/target/front --target=wasm32-unknown-unknown --no-default-features --locked --release
       Front generating JS/WASM with wasm-bindgen
       Using wasm-bindgen version 0.2.108 detected in project
[swc_ecma_transforms_optimization] tree-shaker; pass=0
[swc_ecma_transforms_optimization] tree-shaker; pass=1
[swc_ecma_transforms_optimization] tree-shaker; pass=2
[swc_ecma_transforms_optimization] tree-shaker; pass=0
    Finished generating JS/WASM for front in 4.757749452s
    Tailwind finished tailwindcss --input style/tailwind.css --output /home/develangel/workspace3/rust/keep-it-done/target/tmp/tailwind.css --minify
       
$ file target/site/pkg/kid.wasm
target/site/pkg/kid.wasm: WebAssembly (wasm) binary module version 0x1 (MVP)
```

### Build kid-server

```console
$ export LEPTOS_BIN_CARGO_COMMAND="cross"
$ export LEPTOS_BIN_TARGET_TRIPLE="x86_64-unknown-linux-musl"

$ env | rg LEPTOS
LEPTOS_BIN_CARGO_COMMAND=cross
LEPTOS_BIN_TARGET_TRIPLE=x86_64-unknown-linux-musl

$ cargo leptos build --release --bin-cargo-args="--locked" --lib-cargo-args="--locked"
    Metadata keys ["env", "watch"] from metadata.leptos are not recognized and will be ignored
    Finished `release` profile [optimized] target(s) in 0.28s
       Cargo finished cargo build --package=kid-frontend --lib --target-dir=/var/home/develangel/workspace3/rust/keep-it-done/target/front --target=wasm32-unknown-unknown --no-default-features --locked --release
       Front generating JS/WASM with wasm-bindgen
       Using wasm-bindgen version 0.2.108 detected in project
[swc_ecma_transforms_optimization] tree-shaker; pass=0
[swc_ecma_transforms_optimization] tree-shaker; pass=1
[swc_ecma_transforms_optimization] tree-shaker; pass=2
[swc_ecma_transforms_optimization] tree-shaker; pass=0
    Finished generating JS/WASM for front in 4.758689777s
    Tailwind finished tailwindcss --input style/tailwind.css --output /var/home/develangel/workspace3/rust/keep-it-done/target/tmp/tailwind.css --minify
info: syncing channel updates for 'stable-x86_64-unknown-linux-gnu'

  stable-x86_64-unknown-linux-gnu unchanged - rustc 1.93.1 (01f6ddf75 2026-02-11)

info: checking for self-update
   Compiling ...
    Finished `release` profile [optimized] target(s) in 3m 29s
       Cargo finished cross build --package=kid-server --bin=kid-server --target=x86_64-unknown-linux-musl --no-default-features --locked --release

$ file target/x86_64-unknown-linux-musl/release/kid-server
target/x86_64-unknown-linux-musl/release/kid-server: ELF 64-bit LSB pie executable, x86-64, version 1 (SYSV), static-pie linked, not stripped

$ ldd target/x86_64-unknown-linux-musl/release/kid-server
        statically linked
```

## aarch64 (cross compilation)

Note that the targets `aarch64-unknown-linux-gnu` and `aarch64-unknown-linux-musl`
seem to produce similar results.

### Build kid-cli

```console
$ cross build --target aarch64-unknown-linux-gnu -p kid-cli --release --locked
info: syncing channel updates for 'stable-x86_64-unknown-linux-gnu'

  stable-x86_64-unknown-linux-gnu unchanged - rustc 1.93.1 (01f6ddf75 2026-02-11)

info: checking for self-update
   Compiling ...
    Finished `release` profile [optimized] target(s) in 1m 24s
       Cargo finished cargo build --package=kid-frontend --lib --target-dir=/home/develangel/workspace3/rust/keep-it-done/target/front --target=wasm32-unknown-unknown --no-default-features --locked --release
       Front generating JS/WASM with wasm-bindgen
       Using wasm-bindgen version 0.2.108 detected in project
[swc_ecma_transforms_optimization] tree-shaker; pass=0
[swc_ecma_transforms_optimization] tree-shaker; pass=1
[swc_ecma_transforms_optimization] tree-shaker; pass=2
[swc_ecma_transforms_optimization] tree-shaker; pass=0
    Finished generating JS/WASM for front in 4.757749452s
    Tailwind finished tailwindcss --input style/tailwind.css --output /home/develangel/workspace3/rust/keep-it-done/target/tmp/tailwind.css --minify

$ file target/aarch64-unknown-linux-gnu/release/kid
target/aarch64-unknown-linux-gnu/release/kid: ELF 64-bit LSB pie executable, ARM aarch64, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux-aarch64.so.1, for GNU/Linux 3.7.0, BuildID[sha1]=a1f705f52ea0e20231a3ceba8e4c618f73500fe4, not stripped
```

### Build kid-frontend

```console
$ cargo leptos build --frontend-only --release --lib-cargo-args="--locked"
    Metadata keys ["env", "watch"] from metadata.leptos are not recognized and will be ignored
   Compiling ...
    Finished `release` profile [optimized] target(s) in 2m 23s
       Cargo finished cargo build --package=kid-frontend --lib --target-dir=/home/develangel/workspace3/rust/keep-it-done/target/front --target=wasm32-unknown-unknown --no-default-features --locked --release
       Front generating JS/WASM with wasm-bindgen
       Using wasm-bindgen version 0.2.108 detected in project
[swc_ecma_transforms_optimization] tree-shaker; pass=0
[swc_ecma_transforms_optimization] tree-shaker; pass=1
[swc_ecma_transforms_optimization] tree-shaker; pass=2
[swc_ecma_transforms_optimization] tree-shaker; pass=0
    Finished generating JS/WASM for front in 4.757749452s
    Tailwind finished tailwindcss --input style/tailwind.css --output /home/develangel/workspace3/rust/keep-it-done/target/tmp/tailwind.css --minify
       
$ file target/site/pkg/kid.wasm
target/site/pkg/kid.wasm: WebAssembly (wasm) binary module version 0x1 (MVP)
```

### Build kid-server

```console
$ export LEPTOS_BIN_CARGO_COMMAND="cross"
$ export LEPTOS_BIN_TARGET_TRIPLE="aarch64-unknown-linux-gnu"

$ env | rg LEPTOS
LEPTOS_BIN_CARGO_COMMAND=cross
LEPTOS_BIN_TARGET_TRIPLE=aarch64-unknown-linux-gnu

$ cargo leptos build --release --bin-cargo-args="--locked" --lib-cargo-args="--locked"
    Metadata keys ["env", "watch"] from metadata.leptos are not recognized and will be ignored
    Finished `release` profile [optimized] target(s) in 0.29s
       Cargo finished cargo build --package=kid-frontend --lib --target-dir=/var/home/develangel/workspace3/rust/keep-it-done/target/front --target=wasm32-unknown-unknown --no-default-features --locked --release
       Front generating JS/WASM with wasm-bindgen
       Using wasm-bindgen version 0.2.108 detected in project
[swc_ecma_transforms_optimization] tree-shaker; pass=0
[swc_ecma_transforms_optimization] tree-shaker; pass=1
[swc_ecma_transforms_optimization] tree-shaker; pass=2
[swc_ecma_transforms_optimization] tree-shaker; pass=0
    Finished generating JS/WASM for front in 4.7323659s
    Tailwind finished tailwindcss --input style/tailwind.css --output /var/home/develangel/workspace3/rust/keep-it-done/target/tmp/tailwind.css --minify
info: syncing channel updates for 'stable-x86_64-unknown-linux-gnu'

  stable-x86_64-unknown-linux-gnu unchanged - rustc 1.93.1 (01f6ddf75 2026-02-11)

info: checking for self-update
   Compiling ...
    Finished `release` profile [optimized] target(s) in 4m 17s
       Cargo finished cross build --package=kid-server --bin=kid-server --target=aarch64-unknown-linux-gnu --no-default-features --locked --release

$ file target/aarch64-unknown-linux-gnu/release/kid-server
target/aarch64-unknown-linux-gnu/release/kid-server: ELF 64-bit LSB pie executable, ARM aarch64, version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux-aarch64.so.1, for GNU/Linux 3.7.0, BuildID[sha1]=93f1386c3f51b31beb5dc96391e0195d1a9de6d7, not stripped

$ ldd target/aarch64-unknown-linux-gnu/release/kid-server
        not a dynamic executable
```
