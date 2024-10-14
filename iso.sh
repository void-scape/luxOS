#!/bin/sh

./build.sh
if [ $? -ne 0 ]; then
    exit 1
fi

mkdir -p isodir
mkdir -p isodir/boot
mkdir -p isodir/boot/grub

cp $1 isodir/boot/kernel.bin
cat >isodir/boot/grub/grub.cfg <<EOF
menuentry "Lux" {
	multiboot /boot/kernel.bin
}
EOF
grub-mkrescue -o lux.iso isodir >/dev/null 2>&1
