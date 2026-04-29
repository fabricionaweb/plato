# Build

Start by cloning the repository:

```sh
git clone https://github.com/baskerville/plato.git
cd plato
```

## Plato

#### Preliminary

Install the appropriate [compiler toolchain](https://drive.google.com/drive/folders/1YT6x2X070-cg_E8iWvNUUrWg5-t_YcV0) (the binaries of the `bin` directory need to be in your path).

Install the required dependencies: `wget`, `curl`, `git`, `pkg-config`, `unzip`, `jq`, `patchelf`.

Install *rustup*:
```sh
curl https://sh.rustup.rs -sSf | sh
```

Install the appropriate target:
```sh
rustup target add arm-unknown-linux-gnueabihf
```

#### Notes for MacOS

Toolchain for Apple Silicon Macs (M1/M2) native build [messense/homebrew-macos-cross-toolchains](https://github.com/messense/homebrew-macos-cross-toolchains). Direct download link for [v11.2.0](https://github.com/messense/homebrew-macos-cross-toolchains/releases/download/v11.2.0/arm-unknown-linux-gnueabihf-aarch64-darwin.tar.gz). Newer versions may not work due different glibc versions. I have success with 11.2.0.

For Catalina and later versions, sign the binaries to (hopefully) keep MacOS from blocking execution.

```sh
cd /full/path/to/extracted/toolchain/
find ./ -type f -perm +111 | xargs -n1 sudo codesign --force --deep --sign -
```

Create the following symlinks to compensate for a different naming scheme.

```sh
cd /full/path/to/extracted/toolchain/bin
ln -s arm-unknown-linux-gnueabihf-gcc arm-linux-gnueabihf-gcc
ln -s arm-unknown-linux-gnueabihf-ar arm-linux-gnueabihf-ar
ln -s arm-unknown-linux-gnueabihf-strip arm-linux-gnueabihf-strip
```

Make the "bin" folder avalilable to your `$PATH`


### Build Phase

```sh
./build.sh
```

### Distribution

```sh
./dist.sh
```

## Developer Tools

Install the required dependencies: *MuPDF 1.27.0*, *DjVuLibre*, *FreeType*, *HarfBuzz*.

### Emulator

Install one additional dependency: *SDL2*.

You can then run the emulator with:
```sh
./run-emulator.sh
```

### Importer

You can install the importer with:
```sh
./install-importer.sh
```
