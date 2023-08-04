fn main() {
    cc::Build::new()
        .file("src/console.c")
        .compile("console");
}
