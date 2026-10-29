//! Transport regression: bounded readers retain the existing PCM/IMA decoder.
use std::io::{Cursor, Read, Seek, SeekFrom};
use vera20k_core::assets::wav_file::{WavFile, WavPcmCursor, WavReadCursor, WavStreamHeader};
struct Counted {
    inner: Cursor<Vec<u8>>,
    bytes: usize,
}
impl Read for Counted {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(out)?;
        self.bytes += n;
        Ok(n)
    }
}
impl Seek for Counted {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(pos)
    }
}
fn wave(tag: u16, bits: u16, align: u16, payload: Vec<u8>) -> Vec<u8> {
    let mut bytes = vec![0u8; 44];
    bytes[..4].copy_from_slice(b"RIFF");
    bytes[8..16].copy_from_slice(b"WAVEfmt ");
    bytes[16..20].copy_from_slice(&16u32.to_le_bytes());
    bytes[20..22].copy_from_slice(&tag.to_le_bytes());
    bytes[22..24].copy_from_slice(&1u16.to_le_bytes());
    bytes[24..28].copy_from_slice(&22050u32.to_le_bytes());
    bytes[32..34].copy_from_slice(&align.to_le_bytes());
    bytes[34..36].copy_from_slice(&bits.to_le_bytes());
    bytes[36..40].copy_from_slice(b"data");
    bytes[40..44].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    bytes.extend(payload);
    let size = (bytes.len() - 8) as u32;
    bytes[4..8].copy_from_slice(&size.to_le_bytes());
    bytes
}
fn compare(bytes: Vec<u8>) {
    let wav = WavFile::parse(&bytes).unwrap();
    let mut original = WavPcmCursor::new(&wav).unwrap();
    let mut expected = Vec::new();
    while original.decode_next(wav.data, &mut expected) {}
    let mut reader = Counted {
        inner: Cursor::new(bytes),
        bytes: 0,
    };
    let header = WavStreamHeader::read(&mut reader).unwrap();
    assert_eq!(reader.bytes, 44);
    let mut stream = WavReadCursor::new(header).unwrap();
    let mut actual = Vec::new();
    while stream.read_next(&mut reader, &mut actual).unwrap() {}
    assert_eq!(actual, expected);
}
#[test]
fn pcm_stream_crosses_runs_without_reading_payload_for_metadata() {
    compare(wave(
        1,
        16,
        2,
        (0..17003).map(|n| (n % 251) as u8).collect(),
    ));
}
#[test]
fn ima_stream_preserves_previous_block_bytes_for_short_tail() {
    let mut payload = vec![0; 512 * 2 + 39];
    for (i, value) in payload.iter_mut().enumerate() {
        *value = (i % 13) as u8;
    }
    for p in [0, 512, 1024] {
        payload[p..p + 4].copy_from_slice(&[23, 0, 2, 0]);
    }
    compare(wave(0x11, 4, 512, payload));
}
