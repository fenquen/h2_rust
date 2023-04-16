fn main() {
    cc::Build::new().file("src/h2_rust_common/file_lock.c")
        .compile("lib-file_lock.a");
}