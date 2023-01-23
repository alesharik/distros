fn main() {
    let bios_path = env!("BIOS_PATH");

    let mut child = std::process::Command::new("qemu-system-x86_64")
        // .arg("-gdb").arg("tcp::9002")
        .arg("-machine").arg("q35")
        .arg("-device").arg("nec-usb-xhci,id=xhci")
        .arg("-monitor").arg("stdio")
        .arg("-device").arg("pcie-root-port,id=rp1,slot=1")
        .arg("-device").arg("pcie-pci-bridge,id=br1,bus=rp1")
        .arg("-device").arg("rtl8139,bus=br1,addr=8")
        .arg("-drive").arg(format!("format=raw,file={bios_path}"))
        .spawn()
        .unwrap();
    child.wait().unwrap();
}