build:
    cargo +nightly build -Z build-std=core,alloc --release
size:
    ls -lh target/release/ | sed -n '1,200p' | grep busybox-like.exe
