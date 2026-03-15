use vergen::EmitBuilder;

fn main() {
    // Generate build timestamp
    EmitBuilder::builder()
        .all_build()
        .all_git()
        .emit()
        .unwrap_or_else(|_| {
            // Fallback if git is not available
            println!("cargo:rustc-env=VERGEN_BUILD_TIMESTAMP=unknown");
            println!("cargo:rustc-env=VERGEN_GIT_SHA=unknown");
        });
}
