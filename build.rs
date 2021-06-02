extern crate cc;

fn main() {
    cc::Build::new()
        .include("src/memory")
        .file("src/memory/liballoc.c")
        .compile("liballoc.a");
}