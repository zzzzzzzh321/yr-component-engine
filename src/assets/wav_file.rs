//! Borrowed RIFF/WAVE metadata shared by playback and the retail music list,
//! and the payload decoder both playback paths read it through.
//!
//! Native `408610` supplies the descriptor and declared data length to `408560`;
//! Theme catalog scan `7207F0` stores the resulting whole seconds. Its IMA rate
//! is deliberately different from both the WAV average rate and playback length.

use std::{
    io::{Cursor, Read, Seek, SeekFrom},
    ops::Range,
};

use crate::assets::ima_adpcm::ImaBlockDecoder;

#[derive(Debug, Clone, Copy)]
pub struct WavFile<'a> {
    pub format_tag: u16,
    pub channels: u16,
    pub sample_rate: u32,
    pub block_align: u16,
    pub bits_per_sample: u16,
    pub(crate) declared_data_len: u32,
    pub data: &'a [u8],
    /// File offset of `data`.
    data_start: usize,
}

impl<'a> WavFile<'a> {
    /// Preserve the playback parser's first data-after-fmt choice, bounded data
    /// slice and word-aligned chunk walk. Malformed short fmt chunks fail safely.
    pub fn parse(bytes: &'a [u8]) -> Option<Self> {
        let header = WavStreamHeader::read(&mut Cursor::new(bytes))?;
        Some(Self {
            data_start: header.payload.start,
            data: bytes.get(header.payload)?,
            ..header.format
        })
    }

    /// Original `408751..4087C3` recomputes the descriptor rate, overwriting
    /// nAvgBytesPerSec. IMA uses (2 * channels * sample_rate) / 4; PCM uses
    /// (bits / 8) * channels * sample_rate. `408560` divides data*1000 by that
    /// rate, then `7208D3..7208EC` divides by1000 before storing float seconds.
    /// Ordinary positive audio sizes/rates are covered by sound_theme_metadata.
    pub fn native_duration_seconds(&self) -> Option<u32> {
        let sample_bytes = match self.format_tag {
            1 => u32::from(self.bits_per_sample >> 3),
            0x11 => 2,
            _ => return None,
        };
        let mut rate = sample_bytes
            .checked_mul(u32::from(self.channels))?
            .checked_mul(self.sample_rate)?;
        if self.format_tag == 0x11 {
            rate /= 4;
        }
        if rate == 0 {
            return Some(0);
        }
        let milliseconds = u64::from(self.declared_data_len) * 1000 / u64::from(rate);
        u32::try_from(milliseconds / 1000).ok()
    }

    /// Where [`Self::data`] lies in the parsed file bytes.
    pub fn data_range(&self) -> Range<usize> {
        self.data_start..self.data_start + self.data.len()
    }
}

/// Header-only read through the same RIFF owner as borrowed playback. `format`
/// contains no payload; `payload` identifies the bounded file range to stream.
pub struct WavStreamHeader {
    pub format: WavFile<'static>,
    pub payload: Range<usize>,
}
impl WavStreamHeader {
    pub fn read<R: Read + Seek + ?Sized>(reader: &mut R) -> Option<Self> {
        let length = usize::try_from(reader.seek(SeekFrom::End(0)).ok()?).ok()?;
        if length < 44 {
            return None;
        }
        reader.seek(SeekFrom::Start(0)).ok()?;
        let mut head = [0; 12];
        reader.read_exact(&mut head).ok()?;
        if &head[..4] != b"RIFF" || &head[8..] != b"WAVE" {
            return None;
        }
        let mut offset = 12usize;
        let mut format = None;
        while offset.checked_add(8)? <= length {
            reader.seek(SeekFrom::Start(offset as u64)).ok()?;
            let mut chunk = [0; 8];
            reader.read_exact(&mut chunk).ok()?;
            let size = u32::from_le_bytes(chunk[4..].try_into().ok()?);
            let start = offset + 8;
            let end = start.checked_add(size as usize)?;
            match &chunk[..4] {
                b"fmt " if end <= length => {
                    if size < 16 {
                        return None;
                    }
                    let mut fmt = [0; 16];
                    reader.read_exact(&mut fmt).ok()?;
                    let word = |i| u16::from_le_bytes([fmt[i], fmt[i + 1]]);
                    format = Some((
                        word(0),
                        word(2),
                        u32::from_le_bytes(fmt[4..8].try_into().ok()?),
                        word(12),
                        word(14),
                    ));
                }
                b"data" => {
                    if let Some((format_tag, channels, sample_rate, block_align, bits_per_sample)) =
                        format
                    {
                        return Some(Self {
                            format: WavFile {
                                format_tag,
                                channels,
                                sample_rate,
                                block_align,
                                bits_per_sample,
                                declared_data_len: size,
                                data: &[],
                                data_start: 0,
                            },
                            payload: start..end.min(length),
                        });
                    }
                }
                _ => {}
            }
            offset = end.checked_add((size & 1) as usize)?;
        }
        None
    }
}

/// A bounded source window: one PCM run or one IMA block plus the previous
/// block. The retained prefix preserves the native short-tail stale padding.
pub struct WavReadCursor {
    pub header: WavStreamHeader,
    decoder: WavPcmCursor,
    position: usize,
    stride: usize,
    previous: Vec<u8>,
    input: Vec<u8>,
}
impl WavReadCursor {
    pub fn new(header: WavStreamHeader) -> Option<Self> {
        let decoder = WavPcmCursor::new(&header.format)?;
        let stride = if header.format.format_tag == 0x11 {
            header.format.block_align as usize
        } else {
            PCM_RUN_BYTES
        };
        if stride == 0 {
            return None;
        }
        Some(Self {
            header,
            decoder,
            position: 0,
            stride,
            previous: Vec::new(),
            input: Vec::new(),
        })
    }
    pub fn read_next<R: Read + Seek + ?Sized>(
        &mut self,
        reader: &mut R,
        out: &mut Vec<i16>,
    ) -> std::io::Result<bool> {
        let take = self
            .stride
            .min(self.header.payload.len().saturating_sub(self.position));
        if take == 0 {
            return Ok(false);
        }
        reader.seek(SeekFrom::Start(
            (self.header.payload.start + self.position) as u64,
        ))?;
        self.input.clear();
        self.input.extend_from_slice(&self.previous);
        let prefix = self.input.len();
        self.input.resize(prefix + take, 0);
        reader.read_exact(&mut self.input[prefix..])?;
        let decoded = self.decoder.decode_at(&self.input, prefix, out);
        if self.header.format.format_tag == 0x11 {
            self.previous.clear();
            self.previous.extend_from_slice(&self.input[prefix..]);
        }
        self.position += take;
        Ok(decoded)
    }
}

/// PCM bytes one [`WavPcmCursor::decode_next`] call converts: a VERA streaming
/// granularity, near one retail IMA block's 1017 frames.
const PCM_RUN_BYTES: usize = 4096;

/// A WAV payload decoded a piece per call into interleaved 16-bit samples in
/// the file's own channel layout, so a long file can stream: one IMA ADPCM
/// block per call, or one run of 8- or 16-bit PCM (8-bit widened as
/// `(byte - 128) << 8`).
///
/// `WAV__ParseHeader @ 0x00408610` maps `wFormatTag == 0x11` onto the audio
/// format descriptor the bag index fills, with the fmt chunk's `nBlockAlign`
/// (`psVar6[6]`) as the block stride; `FUN_00409C40 @ 0x00409C40` then installs
/// the one block decoder, `IMA_ADPCM__DecodeBlock @ 0x0040AA70`, so WAV and bag
/// IMA share `ima_adpcm` here as they share it natively.
pub struct WavPcmCursor {
    format: PayloadFormat,
    /// PCM payload offset of the next run.
    pos: usize,
}

enum PayloadFormat {
    Ima(ImaBlockDecoder),
    Pcm16,
    Pcm8,
}

impl WavPcmCursor {
    fn decode_at(&mut self, data: &[u8], position: usize, out: &mut Vec<i16>) -> bool {
        if let PayloadFormat::Ima(decoder) = &mut self.format {
            return decoder.decode_at(data, position, out);
        }
        self.pos = position;
        self.decode_next(data, out)
    }
    /// `None` for a payload playback does not decode: a format tag other than
    /// PCM or IMA ADPCM, or PCM other than 8 or 16 bits.
    pub fn new(wav: &WavFile<'_>) -> Option<Self> {
        match wav.format_tag {
            0x11 => Some(Self {
                format: PayloadFormat::Ima(ImaBlockDecoder::new(
                    wav.channels,
                    u32::from(wav.block_align),
                )),
                pos: 0,
            }),
            1 => Self::pcm(wav.bits_per_sample),
            _ => None,
        }
    }

    /// A cursor over raw PCM of `bits_per_sample`; `None` unless 8 or 16.
    pub fn pcm(bits_per_sample: u16) -> Option<Self> {
        let format = match bits_per_sample {
            16 => PayloadFormat::Pcm16,
            8 => PayloadFormat::Pcm8,
            _ => return None,
        };
        Some(Self { format, pos: 0 })
    }

    /// Append the next piece of `data` — the whole payload, passed again on
    /// every call — to `out`; `false`, appending nothing, once no whole sample
    /// remains.
    pub fn decode_next(&mut self, data: &[u8], out: &mut Vec<i16>) -> bool {
        let sample_bytes = match &mut self.format {
            PayloadFormat::Ima(decoder) => return decoder.decode_next(data, out),
            PayloadFormat::Pcm16 => 2,
            PayloadFormat::Pcm8 => 1,
        };
        let start = self.pos.min(data.len());
        let run = &data[start..(start + PCM_RUN_BYTES).min(data.len())];
        let run = &run[..run.len() / sample_bytes * sample_bytes];
        if run.is_empty() {
            return false;
        }
        self.pos = start + run.len();
        if sample_bytes == 2 {
            out.extend(
                run.as_chunks::<2>()
                    .0
                    .iter()
                    .map(|&c| i16::from_le_bytes(c)),
            );
        } else {
            out.extend(run.iter().map(|&b| (i16::from(b) - 128) << 8));
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_duration_matches_original_instructions_and_retail_headers() {
        let fixture: serde_json::Value = serde_json::from_str(crate::test_fixture::text(
            "tools/storage_oracle/sound_theme_metadata.json",
        ))
        .unwrap();
        let cases = fixture["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 24);
        assert_eq!(
            cases.iter().filter(|case| case["kind"] == "retail").count(),
            8
        );
        for case in cases {
            let header = case["header_hex"].as_str().unwrap();
            let mut bytes: Vec<u8> = (0..header.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&header[i..i + 2], 16).unwrap())
                .collect();
            // Supply the declared ordinary payload; no decode or file I/O is
            // involved. Original fmt/fact bytes remain exactly as recorded.
            bytes.resize(
                bytes.len() + case["data_bytes"].as_u64().unwrap() as usize,
                0,
            );
            let wav = WavFile::parse(&bytes).unwrap();
            assert_eq!(wav.format_tag, case["format_tag"].as_u64().unwrap() as u16);
            assert_eq!(wav.channels, case["channels"].as_u64().unwrap() as u16);
            assert_eq!(
                wav.sample_rate,
                case["sample_rate"].as_u64().unwrap() as u32
            );
            assert_eq!(
                wav.block_align,
                case["block_align"].as_u64().unwrap() as u16
            );
            assert_eq!(
                wav.bits_per_sample,
                case["bits_per_sample"].as_u64().unwrap() as u16
            );
            assert_eq!(
                wav.declared_data_len,
                case["data_bytes"].as_u64().unwrap() as u32
            );
            assert_eq!(
                wav.native_duration_seconds(),
                Some(case["seconds"].as_u64().unwrap() as u32),
                "{}",
                case["name"]
            );
        }
    }
}
