#PKG_CONFIG_ALLOW_CROSS=1 PKG_CONFIG_SYSROOT_DIR=/mnt/e/Projects/raspbianroot PKG_CONFIG_PATH=/mnt/e/Projects/raspbianroot/usr/lib/arm-linux-gnueabihf/pkgconfig cargo build --target=armv7-unknown-linux-gnueabihf


#!/bin/sh



SYSROOT=/build/root



export PKG_CONFIG_DIR=/mnt/e/Projects/raspbianroot 

export PKG_CONFIG_LIBDIR=${SYSROOT}/usr/lib/pkgconfig:${SYSROOT}/usr/share/pkgconfig

export PKG_CONFIG_SYSROOT_DIR=${SYSROOT}

export PKG_CONFIG_ALLOW_CROSS=1



cargo build --target=armv7-unknown-linux-gnueabihf
