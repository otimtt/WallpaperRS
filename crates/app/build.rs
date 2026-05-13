use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/icon.png");

    let src_png = PathBuf::from("../../assets/icon.png");
    if !src_png.exists() {
        println!("cargo:warning=assets/icon.png not found, skipping .ico generation");
        return;
    }

    let out_dir  = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let ico_path = out_dir.join("wallpaper-rs.ico");

    build_ico(&src_png, &ico_path);

    // Embed the .ico into the Windows .exe (Explorer thumbnail + titlebar)
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon(ico_path.to_str().unwrap());
        res.set("ProductName",     "Wallpaper RS");
        res.set("FileDescription", "Wallpaper RS — Motor de Wallpapers Animados");
        res.set("CompanyName",     "WallpaperRS");
        res.set("LegalCopyright",  "© 2026 WallpaperRS");
        res.compile().expect("winres compile failed");
    }

    println!("cargo:rustc-env=WALLPAPER_RS_ICO={}", ico_path.display());
}

fn build_ico(src: &PathBuf, dst: &PathBuf) {
    use image::GenericImageView;

    let img = image::open(src).expect("open icon.png");
    let mut dir = ico::IconDir::new(ico::ResourceType::Icon);

    for size in [16u32, 24, 32, 48, 64, 128, 256] {
        let resized = img.resize(size, size, image::imageops::FilterType::Lanczos3);
        let (w, h)  = resized.dimensions();
        let rgba    = resized.to_rgba8().into_raw();
        let entry   = ico::IconImage::from_rgba_data(w, h, rgba);
        dir.add_entry(ico::IconDirEntry::encode(&entry).expect("ico encode"));
    }

    let mut f = std::fs::File::create(dst).expect("create .ico");
    dir.write(&mut f).expect("write .ico");
}
