use cc::Build;

fn main() {
    Build::new()
        .file("src/stb_image.c")
        .compile("libstb_image.a");
}
