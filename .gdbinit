file guest/target/x86_64-unknown-none/release/guest
target remote :8080
set disassembly-flavor intel
set disassemble-next-line on
enable pretty-printer
layout src
set output-radix 16
