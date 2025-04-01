# xbuild
xbuild is a build tool for rust projects with support for cross compiling and publishing to all
major stores. The goal of xbuild is making native app development as easy as web development.

## Getting started
Install `xbuild`:
```sh
cargo install xbuild
```

List connected devices:
```sh
x devices
host                                              Linux               linux x64           Arch Linux 5.16.10-arch1-1
adb:16ee50bc                                      FP4                 android arm64       Android 11 (API 30)
imd:55abbd4b70af4353bdea2595bbddcac4a2b7891a      David’s iPhone      ios arm64           iPhone OS 15.3.1
```

Build or run on a device:
```sh
x build --device adb:16ee50bc
[1/3] Fetch precompiled artifacts
info: component 'rust-std' for target 'aarch64-linux-android' is up to date
[1/3] Fetch precompiled artifacts [72ms]
[2/3] Build rust
    Finished dev [unoptimized + debuginfo] target(s) in 0.11s
[2/3] Build rust [143ms]
[3/3] Create apk [958ms]
```

![x](https://user-images.githubusercontent.com/741807/162616805-30b48faa-84f0-4fec-851a-4c94fd35c6bd.png)

## Troubleshooting

### Command not found
`x doctor` should let you know if there are any external tools missing.

```sh
 x doctor
--------------------clang/llvm toolchain--------------------
clang                14.0.6              /usr/bin/clang
clang++              14.0.6              /usr/bin/clang++
llvm-ar              unknown             /usr/bin/llvm-ar
llvm-lib             unknown             /usr/bin/llvm-lib
lld                  14.0.6              /usr/bin/lld
lld-link             14.0.6              /usr/bin/lld-link
lldb                 14.0.6              /usr/bin/lldb
lldb-server          unknown             /usr/bin/lldb-server

----------------------------rust----------------------------
rustup               1.25.1              /usr/bin/rustup
cargo                1.64.0              /usr/bin/cargo

--------------------------android---------------------------
adb                  1.0.41              /usr/bin/adb
javac                11.0.17             /usr/bin/javac
java                 11.0.17             /usr/bin/java
kotlin               1.7.20-release-201  /usr/bin/kotlin
gradle               7.5.1               /usr/bin/gradle

----------------------------ios-----------------------------
idevice_id           1.3.0-167-gb314f04  /usr/bin/idevice_id
ideviceinfo          1.3.0-167-gb314f04  /usr/bin/ideviceinfo
ideviceinstaller     1.1.1               /usr/bin/ideviceinstaller
ideviceimagemounter  1.3.0-167-gb314f04  /usr/bin/ideviceimagemounter
idevicedebug         1.3.0-167-gb314f04  /usr/bin/idevicedebug

---------------------------linux----------------------------
mksquashfs           4.5.1               /usr/bin/mksquashfs
```

### error: failed to run custom build command for glib-sys v0.14.0
This means that `gtk3-dev` is not installed. Install `gtk3-dev` package to fix the problem.

### Generating apple signing key/certificate
See [apple_codesign_certificate_management](https://github.com/indygreg/apple-platform-rs/blob/main/apple-codesign/docs/apple_codesign_certificate_management.rst) for further information.

### Generating mobile provisioning profiles
Without an apple developer account there is no cross platform way of generating mobile provisioning
profiles. You can either figure out how to generate it using xcode or [cook](https://github.com/n3d1117/cook).

## License
Apache-2.0 OR MIT
