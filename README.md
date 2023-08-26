# n64-memory-map

This is a small helper utility to quickly lookup where a virtual address lands
in the memory map documented on the N64Brew wiki. It also has functionality to
format instruction traces produced by the Ares emulator, if given a file name.

## Memory Map

See https://n64brew.dev/wiki/Memory_map

# Building

This project is written in [Rust](https://www.rust-lang.org/) and requires the
standard build tools (cargo, rustc, etc). Use this command to build the tool:

```
cargo build --release
```

# Examples

## Virtual Address

If the first argument to the CLI is a 32-bit hexadecimal address prefixed with
"0x", information about its location in the memory map is printed, e.g.

```
$ n64-memory-map 0xB0000000
AddressLocation {
    address: 268435456,
    segment: Some(
        (
            "1",
            "KSEG1",
        ),
    ),
    region: Some(
        (
            "P",
            "PI 1/2",
        ),
    ),
    subregions: [
        (
            "CROM",
            "Cartridge ROM",
        ),
    ],
}
```

```
$ n64-memory-map 0x00FFFFFF
AddressLocation {
    address: 16777215,
    segment: Some(
        (
            "U",
            "KUSEG",
        ),
    ),
    region: Some(
        (
            "R",
            "RDRAM",
        ),
    ),
    subregions: [
        (
            "RDRM",
            "RDRAM memory-space",
        ),
    ],
}
```

## Instruction trace from Ares

An Ares trace log may contain content that looks like this
```
CPU  ffffffffa40005f0  sw      t0{$f0f0f000},v0+$3dd0{$a003e300}
CPU  ffffffffa40005f4  lui     t3,$0000
CPU  ffffffffa40005f8  ori     t3,t3{$00000000},$3303
CPU  ffffffffa40005fc  sw      t3{$00003303},at+$0{$a4400000}
VI I/O: VI_CONTROL <= 00003303
CPU  ffffffffa4000600  sw      t6{$a0002000},at+$4{$a4400004}
VI I/O: VI_DRAM_ADDRESS <= a0002000
CPU  ffffffffa4000604  li      t3,$00000140
CPU  ffffffffa4000608  sw      t3{$00000140},at+$8{$a4400008}
VI I/O: VI_H_WIDTH <= 00000140
CPU  ffffffffa400060c  li      t3,$00000000
CPU  ffffffffa4000610  lui     t3,$03e5
CPU  ffffffffa4000614  ori     t3,t3{$03e50000},$2239
CPU  ffffffffa4000618  sw      t3{$03e52239},at+$14{$a4400014}
VI I/O: VI_TIMING <= 03e52239
CPU  ffffffffa400061c  li      t3,$00000000
CPU  ffffffffa4000620  ori     t3,t3{$00000000},$20d
CPU  ffffffffa4000624  sw      t3{$0000020d},at+$18{$a4400018}
VI I/O: VI_V_SYNC <= 0000020d
CPU  ffffffffa4000628  li      t3,$00000000
CPU  ffffffffa400062c  lui     t3,$0015
```

The CLI can be used to annotate the virtual addresses, e.g.:
```
$ n64-memory-map example.log
CPU 1G.RSPD      0xa40005f0 sw      t0{$f0f0f000},v0+$3dd0{$a003e300}
CPU 1G.RSPD      0xa40005f4 lui     t3,$0000
CPU 1G.RSPD      0xa40005f8 ori     t3,t3{$00000000},$3303
CPU 1G.RSPD      0xa40005fc sw      t3{$00003303},at+$0{$a4400000}
VI I/O: VI_CONTROL <= 00003303
CPU 1G.RSPD      0xa4000600 sw      t6{$a0002000},at+$4{$a4400004}
VI I/O: VI_DRAM_ADDRESS <= a0002000
CPU 1G.RSPD      0xa4000604 li      t3,$00000140
CPU 1G.RSPD      0xa4000608 sw      t3{$00000140},at+$8{$a4400008}
VI I/O: VI_H_WIDTH <= 00000140
CPU 1G.RSPD      0xa400060c li      t3,$00000000
CPU 1G.RSPD      0xa4000610 lui     t3,$03e5
CPU 1G.RSPD      0xa4000614 ori     t3,t3{$03e50000},$2239
CPU 1G.RSPD      0xa4000618 sw      t3{$03e52239},at+$14{$a4400014}
VI I/O: VI_TIMING <= 03e52239
CPU 1G.RSPD      0xa400061c li      t3,$00000000
CPU 1G.RSPD      0xa4000620 ori     t3,t3{$00000000},$20d
CPU 1G.RSPD      0xa4000624 sw      t3{$0000020d},at+$18{$a4400018}
VI I/O: VI_V_SYNC <= 0000020d
CPU 1G.RSPD      0xa4000628 li      t3,$00000000
CPU 1G.RSPD      0xa400062c lui     t3,$0015
```
