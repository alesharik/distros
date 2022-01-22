extern crate cc;

fn main() {
    cc::Build::new()
        .include("src/memory/process")
        .file("src/memory/process/liballoc.c")
        .compile("liballoc.a");
}
