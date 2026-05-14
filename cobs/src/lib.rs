#![cfg_attr(not(test), no_std)]

/**
Example of COBs encoding
00 11 00

00 11 00
00 = 1 byte to 00. Prepend 01. Replace 00 with 02 (2 bytes to next 00) <- we don't know where the next 0 is yet
11 -> stays <- should be written into the new buffer directly
00 -> becomes 01 <- 01 points to the terminator byte
append 00 <- this is the terminator byte

result 01 02 11 01 00
*/

/// Encodes `data[..length]` into `buffer` using COBS. Returns the number of bytes written,
/// including the trailing `0x00` delimiter.
/// Based on https://en.wikipedia.org/wiki/Consistent_Overhead_Byte_Stuffing
pub fn encode(data: &[u8], length: usize, buffer: &mut [u8]) -> usize {
    let mut write_pos: usize = 0; // current buffer write position
    let mut overhead_pos = write_pos; // reserved slot for the current block's overhead byte
    let mut non_zero_count = 1u8; // distance to next overhead: 1 + non-zero bytes written since last overhead
    write_pos += 1;

    for i in 0..length {
        // non-zero bytes written directly into write_pos
        if data[i] != 0 {
            buffer[write_pos] = data[i];
            write_pos += 1;
            non_zero_count += 1; // increment non-zero
        }

        // if zero byte or block is complete
        if data[i] == 0 || non_zero_count == 0xFF {
            buffer[overhead_pos] = non_zero_count; // write to the last overhead position the non-zero bytes
            non_zero_count = 1; // reset count
            overhead_pos = write_pos; // reserve current position as overhead slot for next block
            // increment write_pos if we aren't at the end of the block
            if data[i] == 0 || i < length - 1 {
                write_pos += 1;
            }
        }
    }
    buffer[overhead_pos] = non_zero_count;

    buffer[write_pos] = 0x00;
    write_pos + 1
}

/// Decodes a COBS-encoded `buffer[..length]` into `data`. Returns the number of decoded bytes.
pub fn decode(buffer: &[u8], length: usize, data: &mut [u8]) -> usize {
    let mut next_overhead = buffer[0] as usize; // in bytes
    let mut last_overhead = buffer[0];
    let mut data_len = 0usize;

    for i in 1..length {
        if i == next_overhead {
            if buffer[i] == 0x00 {
                break;
            }
            // 0xFF overhead means block ended at the 254-byte limit, not at a zero
            if last_overhead != 0xFF {
                data[data_len] = 0x00;
                data_len += 1;
            }
            last_overhead = buffer[i];
            next_overhead = i + buffer[i] as usize; // i as offset + bytes to next
        } else if buffer[i] != 0x00 {
            data[data_len] = buffer[i];
            data_len += 1;
        } else {
            break;
        }
    }

    data_len
}

#[cfg(test)]
mod tests {
    /// Test examples from https://en.wikipedia.org/wiki/Consistent_Overhead_Byte_Stuffing
    use crate::{decode, encode};

    #[test]
    fn test_row_1() {
        let data = [0x00u8];
        let mut buf = [0x00u8; 8];
        let encoded_size = encode(&data, data.len(), &mut buf);

        let expected = [0x01u8, 0x01, 0x00];
        println!("data: {:02X?}", &data);
        println!("buf:  {:02X?}", &buf[..encoded_size]);
        assert_eq!(buf[0..encoded_size], expected);
    }

    #[test]
    fn test_row_2() {
        let data = [0x00u8, 0x00];
        let mut buf = [0x00u8; 8];
        let encoded_size = encode(&data, data.len(), &mut buf);

        let expected = [0x01, 0x01, 0x01, 0x00];
        println!("data: {:02X?}", &data);
        println!("buf:  {:02X?}", &buf[..encoded_size]);
        assert_eq!(buf[0..encoded_size], expected);
    }

    #[test]
    fn test_row_3() {
        let data = [0x00u8, 0x11, 0x00];
        let mut buf = [0x00u8; 8];
        let encoded_size = encode(&data, data.len(), &mut buf);

        let expected = [0x01u8, 0x02, 0x11, 0x01, 0x00];
        println!("data: {:02X?}", &data);
        println!("buf:  {:02X?}", &buf[..encoded_size]);
        assert_eq!(buf[0..encoded_size], expected);
    }

    #[test]
    fn test_row_4() {
        let data = [0x11, 0x22, 0x00, 0x33];
        let mut buf = [0x00u8; 8];
        let encoded_size = encode(&data, data.len(), &mut buf);

        let expected = [0x03, 0x11, 0x22, 0x02, 0x33, 0x00];
        println!("data: {:02X?}", &data);
        println!("buf:  {:02X?}", &buf[..encoded_size]);
        assert_eq!(buf[0..encoded_size], expected);
    }

    #[test]
    fn test_row_5() {
        let data = [0x11, 0x22, 0x33, 0x44];
        let mut buf = [0x00u8; 8];
        let encoded_size = encode(&data, data.len(), &mut buf);

        let expected = [0x05, 0x11, 0x22, 0x33, 0x44, 0x00];
        println!("data: {:02X?}", &data);
        println!("buf:  {:02X?}", &buf[..encoded_size]);
        assert_eq!(buf[0..encoded_size], expected);
    }

    #[test]
    fn test_row_6() {
        // 11 00 00 00 → 02 11 01 01 01 00
        let data = [0x11, 0x00, 0x00, 0x00];
        let mut buf = [0u8; 8];
        let encoded_size = encode(&data, data.len(), &mut buf);

        let expected = [0x02, 0x11, 0x01, 0x01, 0x01, 0x00];
        println!("buf:      {:02X?}", &buf[..encoded_size]);
        println!("expected: {:02X?}", &expected);
        assert_eq!(buf[..encoded_size], expected);
    }

    #[test]
    fn test_row_7() {
        let mut data: [u8; 254] = [0x00; 254];
        for i in 0x01..0xFFu8 {
            data[i as usize - 1] = i;
        }

        let mut buf = [0x00u8; 256];
        let encoded_size = encode(&data, data.len(), &mut buf);

        let mut expected: [u8; 256] = [0; 256];
        expected[0] = 0xFF;
        for i in 0x01..=0xFFu8 {
            expected[i as usize] = i;
        }
        expected[255] = 00;

        println!("data: {:02X?}", &data);
        println!("buf:  {:02X?}", &buf[..encoded_size]);
        assert_eq!(buf[0..encoded_size], expected);
    }

    #[test]
    fn test_row_8() {
        // 00 01 02 ... FC FD FE → 01 FF 01 02 ... FC FD FE 00
        let mut data = [0u8; 255];
        for i in 0u8..=0xFE {
            data[i as usize] = i;
        }
        let mut buf = [0u8; 512];
        let encoded_size = encode(&data, data.len(), &mut buf);

        let mut expected = [0u8; 257];
        expected[0] = 0x01;
        expected[1] = 0xFF;
        for i in 1u8..=0xFE {
            expected[i as usize + 1] = i;
        }
        expected[256] = 0x00;

        println!("buf:      {:02X?}", &buf[..encoded_size]);
        println!("expected: {:02X?}", &expected);
        assert_eq!(buf[..encoded_size], expected);
    }

    #[test]
    fn test_row_9() {
        // 01 02 03 ... FD FE FF → FF 01 02 03 ... FD FE 02 FF 00
        let mut data = [0u8; 255];
        for i in 0x01u8..=0xFF {
            data[(i - 1) as usize] = i;
        }
        let mut buf = [0u8; 512];
        let encoded_size = encode(&data, data.len(), &mut buf);

        let mut expected = [0u8; 258];
        expected[0] = 0xFF;
        for i in 1u8..=0xFE {
            expected[i as usize] = i;
        }
        expected[255] = 0x02;
        expected[256] = 0xFF;
        expected[257] = 0x00;

        println!("buf:      {:02X?}", &buf[..encoded_size]);
        println!("expected: {:02X?}", &expected);
        assert_eq!(buf[..encoded_size], expected);
    }

    #[test]
    fn test_row_10() {
        // 02 03 04 ... FE FF 00 → FF 02 03 04 ... FE FF 01 01 00
        let mut data = [0u8; 255];
        for i in 0x02u8..=0xFF {
            data[(i - 2) as usize] = i;
        }
        data[254] = 0x00;
        let mut buf = [0u8; 512];
        let encoded_size = encode(&data, data.len(), &mut buf);

        let mut expected = [0u8; 258];
        expected[0] = 0xFF;
        for i in 0x02u8..=0xFF {
            expected[(i - 1) as usize] = i;
        }
        expected[255] = 0x01;
        expected[256] = 0x01;
        expected[257] = 0x00;

        println!("buf:      {:02X?}", &buf[..encoded_size]);
        println!("expected: {:02X?}", &expected);
        assert_eq!(buf[..encoded_size], expected);
    }

    #[test]
    fn test_row_11() {
        // 03 04 05 ... FF 00 01 → FE 03 04 05 ... FF 02 01 00
        let mut data = [0u8; 255];
        for i in 0x03u8..=0xFF {
            data[(i - 3) as usize] = i;
        }
        data[253] = 0x00;
        data[254] = 0x01;
        let mut buf = [0u8; 512];
        let encoded_size = encode(&data, data.len(), &mut buf);

        let mut expected = [0u8; 257];
        expected[0] = 0xFE;
        for i in 0x03u8..=0xFF {
            expected[(i - 2) as usize] = i;
        }
        expected[254] = 0x02;
        expected[255] = 0x01;
        expected[256] = 0x00;

        println!("buf:      {:02X?}", &buf[..encoded_size]);
        println!("expected: {:02X?}", &expected);
        assert_eq!(buf[..encoded_size], expected);
    }

    #[test]
    fn test_row_1_decode() {
        let mut data = [0x00u8; 8];
        let buf = [0x01u8, 0x01, 0x00];
        let expected = [0x00];
        let actual_len = decode(&buf, buf.len(), &mut data);
        assert_eq!(data[0..actual_len], expected);
    }

    #[test]
    fn test_row_2_decode() {
        // 01 01 01 00 → 00 00
        let mut data = [0x00u8; 8];
        let buf = [0x01u8, 0x01, 0x01, 0x00];
        let expected = [0x00, 0x00];
        let actual_len = decode(&buf, buf.len(), &mut data);
        assert_eq!(data[0..actual_len], expected);
    }

    #[test]
    fn test_row_3_decode() {
        // 01 02 11 01 00 → 00 11 00
        let mut data = [0x00u8; 8];
        let buf = [0x01u8, 0x02, 0x11, 0x01, 0x00];
        let expected = [0x00, 0x11, 0x00];
        let actual_len = decode(&buf, buf.len(), &mut data);
        assert_eq!(data[0..actual_len], expected);
    }

    #[test]
    fn test_row_4_decode() {
        // 03 11 22 02 33 00 → 11 22 00 33
        let mut data = [0x00u8; 8];
        let buf = [0x03u8, 0x11, 0x22, 0x02, 0x33, 0x00];
        let expected = [0x11u8, 0x22, 0x00, 0x33];
        let actual_len = decode(&buf, buf.len(), &mut data);
        assert_eq!(data[0..actual_len], expected);
    }

    #[test]
    fn test_row_5_decode() {
        // 05 11 22 33 44 00 → 11 22 33 44
        let mut data = [0x00u8; 8];
        let buf = [0x05u8, 0x11, 0x22, 0x33, 0x44, 0x00];
        let expected = [0x11u8, 0x22, 0x33, 0x44];
        let actual_len = decode(&buf, buf.len(), &mut data);
        assert_eq!(data[0..actual_len], expected);
    }

    #[test]
    fn test_row_6_decode() {
        // 02 11 01 01 01 00 → 11 00 00 00
        let mut data = [0x00u8; 8];
        let buf = [0x02u8, 0x11, 0x01, 0x01, 0x01, 0x00];
        let expected = [0x11u8, 0x00, 0x00, 0x00];
        let actual_len = decode(&buf, buf.len(), &mut data);
        assert_eq!(data[0..actual_len], expected);
    }

    #[test]
    fn test_row_7_decode() {
        // FF 01 02 ... FE 00 → 01 02 ... FE
        let mut encoded = [0x00u8; 256];
        encoded[0] = 0xFF;
        for i in 0x01u8..=0xFE {
            encoded[i as usize] = i;
        }
        encoded[255] = 0x00;

        let mut data = [0x00u8; 254];
        let actual_len = decode(&encoded, encoded.len(), &mut data);

        let mut expected = [0x00u8; 254];
        for i in 0x01u8..=0xFE {
            expected[i as usize - 1] = i;
        }
        assert_eq!(data[0..actual_len], expected);
    }

    #[test]
    fn test_row_8_decode() {
        // 01 FF 01 02 ... FE 00 → 00 01 02 ... FE
        let mut encoded = [0x00u8; 257];
        encoded[0] = 0x01;
        encoded[1] = 0xFF;
        for i in 1u8..=0xFE {
            encoded[i as usize + 1] = i;
        }
        encoded[256] = 0x00;

        let mut data = [0x00u8; 255];
        let actual_len = decode(&encoded, encoded.len(), &mut data);

        let mut expected = [0x00u8; 255];
        for i in 0u8..=0xFE {
            expected[i as usize] = i;
        }
        assert_eq!(data[0..actual_len], expected);
    }

    #[test]
    fn test_row_9_decode() {
        // FF 01 02 ... FD FE 02 FF 00 → 01 02 ... FE FF
        let mut encoded = [0x00u8; 258];
        encoded[0] = 0xFF;
        for i in 1u8..=0xFE {
            encoded[i as usize] = i;
        }
        encoded[255] = 0x02;
        encoded[256] = 0xFF;
        encoded[257] = 0x00;

        let mut data = [0x00u8; 255];
        let actual_len = decode(&encoded, encoded.len(), &mut data);

        let mut expected = [0x00u8; 255];
        for i in 0x01u8..=0xFF {
            expected[(i - 1) as usize] = i;
        }
        assert_eq!(data[0..actual_len], expected);
    }

    #[test]
    fn test_row_10_decode() {
        // FF 02 03 ... FF 01 01 00 → 02 03 ... FF 00
        let mut encoded = [0x00u8; 258];
        encoded[0] = 0xFF;
        for i in 0x02u8..=0xFF {
            encoded[(i - 1) as usize] = i;
        }
        encoded[255] = 0x01;
        encoded[256] = 0x01;
        encoded[257] = 0x00;

        let mut data = [0x00u8; 255];
        let actual_len = decode(&encoded, encoded.len(), &mut data);

        let mut expected = [0x00u8; 255];
        for i in 0x02u8..=0xFF {
            expected[(i - 2) as usize] = i;
        }
        expected[254] = 0x00;
        assert_eq!(data[0..actual_len], expected);
    }

    #[test]
    fn test_row_11_decode() {
        // FE 03 04 ... FF 02 01 00 → 03 04 ... FF 00 01
        let mut encoded = [0x00u8; 257];
        encoded[0] = 0xFE;
        for i in 0x03u8..=0xFF {
            encoded[(i - 2) as usize] = i;
        }
        encoded[254] = 0x02;
        encoded[255] = 0x01;
        encoded[256] = 0x00;

        let mut data = [0x00u8; 255];
        let actual_len = decode(&encoded, encoded.len(), &mut data);

        let mut expected = [0x00u8; 255];
        for i in 0x03u8..=0xFF {
            expected[(i - 3) as usize] = i;
        }
        expected[253] = 0x00;
        expected[254] = 0x01;
        assert_eq!(data[0..actual_len], expected);
    }

    #[test]
    fn test_two_blocks_encode() {
        // 260 × 0x42 → FF [42 × 254] 07 [42 × 6] 00
        // block 1: 254 non-zero bytes hit the limit → overhead 0xFF
        // block 2: 6 remaining bytes → overhead 0x07 (6 + 1)
        let data = [0x42u8; 260];
        let mut buf = [0x00u8; 263];
        let encoded_size = encode(&data, data.len(), &mut buf);

        let mut expected = [0x00u8; 263];
        expected[0] = 0xFF;
        for i in 1..=254 {
            expected[i] = 0x42;
        }
        expected[255] = 0x07;
        for i in 256..=261 {
            expected[i] = 0x42;
        }
        expected[262] = 0x00;

        assert_eq!(buf[..encoded_size], expected);
    }

    #[test]
    fn test_two_blocks_decode() {
        // FF [42 × 254] 07 [42 × 6] 00 → 260 × 0x42
        let mut encoded = [0x00u8; 263];
        encoded[0] = 0xFF;
        for i in 1..=254 {
            encoded[i] = 0x42;
        }
        encoded[255] = 0x07;
        for i in 256..=261 {
            encoded[i] = 0x42;
        }
        encoded[262] = 0x00;

        let mut data = [0x00u8; 260];
        let actual_len = decode(&encoded, encoded.len(), &mut data);

        assert_eq!(actual_len, 260);
        assert_eq!(data, [0x42u8; 260]);
    }
}
