# cobs-encoding

A `no_std`-compatible implementation of [Consistent Overhead Byte Stuffing (COBS)](https://en.wikipedia.org/wiki/Consistent_Overhead_Byte_Stuffing) in Rust.

COBS encodes arbitrary byte data so the output contains no `0x00` bytes, allowing a single `0x00` to be used as a reliable packet delimiter in a byte stream.

## Usage

```rust
use cobs_encoding::{encode, decode};

let data = [0x11, 0x00, 0x22];
let mut encoded = [0u8; 16];
let mut decoded = [0u8; 16];

let encoded_len = encode(&data, data.len(), &mut encoded);
// encoded[..encoded_len] = [0x02, 0x11, 0x02, 0x22, 0x00]

let decoded_len = decode(&encoded, encoded_len, &mut decoded);
assert_eq!(&decoded[..decoded_len], &data);
```

## API

```rust
/// Encodes data[..length] into buffer. Returns bytes written, including the trailing 0x00 delimiter.
pub fn encode(data: &[u8], length: usize, buffer: &mut [u8]) -> usize;

/// Decodes a COBS-encoded buffer[..length] into data. Returns the number of decoded bytes.
pub fn decode(buffer: &[u8], length: usize, data: &mut [u8]) -> usize;
```

## Overhead

Encoding adds at most one overhead byte per 254 bytes of input, plus a single trailing `0x00` delimiter:

```
max encoded size = length + ceil(length / 254) + 1
```

## How it works

The encoded stream is divided into blocks of up to 254 non-zero bytes. Each block is preceded by an overhead byte whose value is the distance to the next overhead byte (or the terminator). Zero bytes in the original data are represented implicitly — the overhead byte at each block boundary signals that a zero was present. A `0xFF` overhead byte means the block hit the 254-byte limit with no zero in the data.

```
Original:  00 11 00
Encoded:   01 02 11 01 00
            │  │      │  └─ terminator
            │  │      └──── overhead: 1 byte to terminator (represents trailing 00)
            │  └─────────── 0x11 (non-zero, copied as-is)
            └────────────── overhead: 1 byte to next overhead (represents leading 00)
```

## Tests

Full test suite covering all 11 example rows from the Wikipedia COBS article, for both encode and decode, plus multi-block tests for data exceeding 254 bytes.
