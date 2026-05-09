## Resources used
- https://www.youtube.com/watch?v=TOAynddiu5M
- https://docs.rust-embedded.org/book/start/hardware.html
- find supported chip for `cargo embed` / proto-rs https://probe.rs/targets/?manufacturer=SHOW_ALL_MANUFACTURERS&family=SHOW_ALL_FAMILIES

## gen info
- Using stm32 Nucleo: this guy here https://www.st.com/en/evaluation-tools/nucleo-f103rb.html#overview
- openocd command is `openocd -f interface/stlink.cfg -f target/stm32f1x.cfg `
- `Embed.toml` is used for `cargo embed` 

## board info
- STM32F103RB
- ARM Cortex-M3
- 

## Commands Used (from tutorials)
```
# print elf headers
cargo readobj --bin hellostm32 -- --file-headers   
# get binary size (for release). Shows linker sections
cargo size --bin hellostm32 --release -- -A    
# disassemble the binary
cargo objdump --bin hellostm32 --release -- --disassemble --no-show-raw-insn --print-imm-hex  
```

connecting gdb
```
hellostm32  > arm-none-eabi-gdb -q target/thumbv7m-none-eabi/debug/hellostm32                                                                                                                      18:15:53
Reading symbols from target/thumbv7m-none-eabi/debug/hellostm32...
(gdb) target remote :3333
Remote debugging using :3333
```

installing cargo embed (now in proto-rs)
```
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh | sh
```

## misc notes
https://docs.rust-embedded.org/book/start/qemu.html
A refresher on ELF linker sections

.text contains the program instructions
.rodata contains constant values like strings
.data contains statically allocated variables whose initial values are not zero
.bss also contains statically allocated variables whose initial values are zero
.vector_table is a non-standard section that we use to store the vector (interrupt) table
.ARM.attributes and the .debug_* sections contain metadata and will not be loaded onto the target when flashing the binary.