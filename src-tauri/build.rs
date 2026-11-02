fn main() {
    println!("cargo:rerun-if-changed=.env");
    if let Ok(iter) = dotenvy::from_filename_iter(".env") {
        for item in iter.flatten() {
            if item.0 == "DATABASE_URL" {
                println!("cargo:rustc-env=DATABASE_URL={}", item.1);
            }
        }
    }

    tauri_build::build()
}