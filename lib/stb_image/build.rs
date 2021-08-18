use cc::Build;

fn main() {
    Build::new()
        .file("src/stb_image.c")
        .flag("-O3")
        .compile("libstb_image.a");
}
