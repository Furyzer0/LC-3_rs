fn main() {
    cc::Build::new()
        .file("C/platform.c")
        .compile("libplatform.a");
}