# w25qxx

A `no_std` driver and simulator for the W25Q family of SPI flash chips (W25Q32, W25Q64, W25Q128, etc.).

## Design

The `Flash` trait defines the core operations. Driver logic is written against the trait, so it works against both the `MockFlash` simulator and a real SPI backend.

```rust
pub trait Flash {
    fn read(&self, addr: u32, buf: &mut [u8]);
    fn write(&mut self, addr: u32, data: &[u8]);
    fn erase_sector(&mut self, addr: u32);
}
```

## MockFlash

`MockFlash<const CAPACITY: usize>` simulates flash hardware constraints in memory:

- **Write** can only flip bits from `1` → `0` (`&=`). Writing to a non-erased location silently corrupts data, matching real hardware behavior.
- **Erase** resets an entire 4KB sector to `0xFF`. Minimum erase unit — individual pages cannot be erased.
- **Read** has no size or alignment constraints. Panics if the read exceeds capacity (real hardware wraps around to `0x000000`).

```rust
use w25qxx::{Flash, MockFlash, SECTOR_SIZE};

const N_SECTORS: usize = 16;
let mut flash = MockFlash::<{ SECTOR_SIZE * N_SECTORS }>::new();

flash.erase_sector(0x0000);
flash.write(0x0000, &[0xDE, 0xAD, 0xBE, 0xEF]);

let mut buf = [0u8; 4];
flash.read(0x0000, &mut buf);
assert_eq!(buf, [0xDE, 0xAD, 0xBE, 0xEF]);
```

## Hardware constraints

| Property    | Value  | Notes                                  |
|-------------|--------|----------------------------------------|
| Page size   | 256 B  | Maximum write unit                     |
| Sector size | 4 KB   | Minimum erase unit (16 pages)          |
| Block size  | 64 KB  | 16 sectors — `erase_block` not yet implemented |
| Endurance   | 100,000 erase cycles per sector        |

## Flash memory model

Flash can only flip bits `1` → `0` on write. Flipping bits back to `1` requires erasing the entire sector. This means writing to an already-written location requires a read-modify-erase-write cycle:

1. Read the full 4KB sector into RAM
2. Modify the target bytes in the buffer
3. Erase the sector
4. Write all pages back

## Next steps

- Implement `W25Q128` SPI backend (`impl Flash for W25Q128<SPI>`)
- Add `erase_block` (64KB)
