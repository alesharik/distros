use std::path::PathBuf;
use std::process::{Command, Stdio};

fn main() {
    Command::new("cargo")
        .arg("+nightly-2024-01-16")
        .arg("build")
        .arg("--target")
        .arg("x86_64-unknown-none")
        .current_dir("..")
        .status()
        .unwrap();

    let kernel = PathBuf::from("../target/x86_64-unknown-none/debug/distros");
    let out_dir = PathBuf::from("target");
    let uefi_path = out_dir.join("uefi.img");
    bootloader::UefiBoot::new(&kernel).create_disk_image(&uefi_path).unwrap();
    let uefi_path = uefi_path.to_string_lossy();

    let mut child = Command::new("qemu-system-x86_64")
        .arg("-gdb").arg("tcp::9002")
        .arg("-no-reboot").arg("-no-shutdown")
        // .arg("-machine").arg("q35")
        // .arg("-device").arg("nec-usb-xhci,id=xhci")
        // .arg("-monitor").arg("stdio")
        // .arg("-device").arg("pcie-root-port,id=rp1,slot=1")
        // .arg("-device").arg("pcie-pci-bridge,id=br1,bus=rp1")
        // .arg("-device").arg("rtl8139,bus=br1,addr=8")
        .arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi())
        .arg("-drive").arg(format!("format=raw,file={uefi_path}"))
        // .arg("-drive").arg(format!("format=raw,file={bios_path}"))
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    child.wait().unwrap();
}