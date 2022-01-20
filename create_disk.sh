#!/bin/sh
if [ -z "$1" ]; then
    profile="debug"
else
    profile="release"
fi;

mkdir _efi/EFI/Boot/ -p && cp target/x86_64-unknown-uefi/$profile/memory.efi _efi/EFI/Boot/Bootx64.efi
virt-make-fs --type=vfat --size=24M _efi efi.raw
qemu-img convert -O vmdk efi.raw efi.vmdk