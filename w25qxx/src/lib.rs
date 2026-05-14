#![cfg_attr(not(test), no_std)]

pub const SECTOR_SIZE: usize = 0x1000; // 4KB, fixed for all W25Q chips
pub const PAGE_SIZE: usize = 256; // fixed for all W25Q chips

pub trait Flash {
    fn read(&self, addr: u32, buf: &mut [u8]);
    fn write(&mut self, addr: u32, data: &[u8]);
    fn erase_sector(&mut self, addr: u32);
}

// CAPACITY = SECTOR_SIZE * N_SECTORS. Rust stable doesn't support const generic arithmetic
// in array sizes, so total capacity is passed directly. e.g. MockFlash<{ SECTOR_SIZE * 16 }>
pub struct MockFlash<const CAPACITY: usize> {
    mem: [u8; CAPACITY],
}

impl<const CAPACITY: usize> MockFlash<CAPACITY> {
    /// Returns a `MockFlash` with all bytes initialized to `0xFF` (erased state).
    pub fn new() -> Self {
        Self { mem: [0xFF; CAPACITY] }
    }
}

impl<const CAPACITY: usize> Flash for MockFlash<CAPACITY> {
    // No limits on read. It can read beyond sector boundaries
    fn read(&self, addr: u32, buf: &mut [u8]) {
        // on W25QXX this would silently wrap back to the starting address
        assert!(addr as usize + buf.len() <= CAPACITY, "read exceeds capacity");
        buf.copy_from_slice(&self.mem[addr as usize..(addr as usize + buf.len())])
    }

    // on flash, we can only go from a 1 -> 0. You can't write a 0 -> 1, which is why erasing first is necessary
    fn write(&mut self, addr: u32, data: &[u8]) {
        // on W25QXX this would silently wrap back to the start of the page
        assert!(
            addr % PAGE_SIZE as u32 + (data.len() as u32) < PAGE_SIZE as u32,
            "data exceeded page size"
        );
        for (i, byte) in data.iter().enumerate() {
            self.mem[addr as usize + i] &= byte; // mimics hardware behavior. Only 1 in mem can go to 0
        }
    }

    // fills the entire sector to 0xFF
    fn erase_sector(&mut self, addr: u32) {
        let sector_start: usize = (addr as usize / SECTOR_SIZE) * SECTOR_SIZE; // get sector (division truncates) then get sector address
        self.mem[sector_start..sector_start + SECTOR_SIZE].fill(0xFF);
    }
}

#[cfg(test)]
mod tests {
    use crate::{Flash, MockFlash, PAGE_SIZE, SECTOR_SIZE};

    const N_SECTORS: usize = 16;
    type TestFlash = MockFlash<{ SECTOR_SIZE * N_SECTORS }>;

    const SECTOR_0: usize = 0x00_0000;
    const SECTOR_1: usize = SECTOR_SIZE;
    const SECTOR_2: usize = SECTOR_SIZE * 2;

    #[test]
    fn erase_sector_0_works() {
        let mut mf = TestFlash {
            mem: [0x42; SECTOR_SIZE * N_SECTORS], // set to arbitrary bytes. Normally would be 0xFF
        };

        mf.erase_sector(SECTOR_0 as u32);

        assert_eq!(mf.mem[SECTOR_0..SECTOR_SIZE], [0xFF; SECTOR_SIZE]);
        assert_eq!(
            mf.mem[SECTOR_1..SECTOR_1 + (SECTOR_SIZE * 15)],
            [0x42; SECTOR_SIZE * 15]
        );
    }

    #[test]
    fn erase_sector_1_works() {
        let mut mf = TestFlash {
            mem: [0x42; SECTOR_SIZE * N_SECTORS], // set to arbitrary bytes. Normally would be 0xFF
        };

        mf.erase_sector(SECTOR_1 as u32);

        assert_eq!(mf.mem[SECTOR_1..SECTOR_1 + SECTOR_SIZE], [0xFF; SECTOR_SIZE]);
        assert_eq!(mf.mem[SECTOR_0..SECTOR_SIZE], [0x42; SECTOR_SIZE]);
        assert_eq!(
            mf.mem[SECTOR_2..SECTOR_2 + (SECTOR_SIZE * 14)],
            [0x42; SECTOR_SIZE * 14]
        );
    }

    #[test]
    fn write_sector_0_works() {
        let mut mf = TestFlash::new();

        const D_LEN: usize = 4;
        let data = [0xDE, 0xAD, 0xBE, 0xEF];
        mf.write(SECTOR_0 as u32, &data);
        assert_eq!(mf.mem[SECTOR_0..D_LEN], data);
        assert_eq!(mf.mem[SECTOR_0 + D_LEN..SECTOR_SIZE], [0xFF; SECTOR_SIZE - D_LEN])
    }

    #[test]
    fn write_sector_1_works() {
        let mut mf = TestFlash::new();

        const D_LEN: usize = 4;
        let data = [0xDE, 0xAD, 0xBE, 0xEF];
        mf.write(SECTOR_1 as u32, &data);

        assert_eq!(mf.mem[SECTOR_1..SECTOR_1 + D_LEN], data);
        assert_eq!(mf.mem[SECTOR_1 + D_LEN..SECTOR_2], [0xFF; SECTOR_SIZE - D_LEN]);
        assert_eq!(mf.mem[SECTOR_0..SECTOR_1], [0xFF; SECTOR_SIZE]);
    }

    #[test]
    fn write_non_zero_start_works() {
        let mut mf = TestFlash::new();

        const OFFSET: usize = SECTOR_SIZE / 2; // 0x800
        const D_LEN: usize = 4;
        let data = [0xDE, 0xAD, 0xBE, 0xEF];
        mf.write((SECTOR_0 + OFFSET) as u32, &data);

        assert_eq!(mf.mem[SECTOR_0..SECTOR_0 + OFFSET], [0xFF; OFFSET]);
        assert_eq!(mf.mem[SECTOR_0 + OFFSET..SECTOR_0 + OFFSET + D_LEN], data);
        assert_eq!(mf.mem[SECTOR_0 + OFFSET + D_LEN..SECTOR_1], [0xFF; OFFSET - D_LEN]);
    }

    #[test]
    #[should_panic]
    fn write_exceeds_page_panics() {
        let mut mf = TestFlash::new();
        let data = [0xAA; PAGE_SIZE + 1];
        mf.write(SECTOR_0 as u32, &data);
    }

    #[test]
    fn read_works() {
        let mut mf = TestFlash::new();

        let data = [0xDE, 0xAD, 0xBE, 0xEF];
        mf.write(SECTOR_0 as u32, &data);

        let mut buf = [0u8; 4];
        mf.read(SECTOR_0 as u32, &mut buf);
        assert_eq!(buf, data);
    }
}
