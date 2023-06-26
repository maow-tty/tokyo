use std::env;
use std::path::PathBuf;
use bootloader::DiskImageBuilder;

fn main() {
    let kernel_path = env::var("CARGO_BIN_FILE_KERNEL").unwrap();
    let image_builder = DiskImageBuilder::new(PathBuf::from(kernel_path));

    let out_dir = PathBuf::from("target").join(env::var("PROFILE").unwrap());
    let uefi_path = out_dir.join("tokyo-uefi.img");
    let bios_path = out_dir.join("tokyo-bios.img");

    image_builder.create_uefi_image(&uefi_path).unwrap();
    image_builder.create_bios_image(&bios_path).unwrap();

    println!("cargo:rustc-env=UEFI_IMAGE={}", uefi_path.display());
    println!("cargo:rustc-env=BIOS_IMAGE={}", bios_path.display());
}