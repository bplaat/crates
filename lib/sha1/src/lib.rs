/*
 * Copyright (c) 2025-2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A minimal replacement for the [sha1](https://crates.io/crates/sha1) crate

/// SHA-1 hasher
pub struct Sha1 {
    state: [u32; 5],
    buffer: [u8; 64],
    length: u64,
}

impl Default for Sha1 {
    fn default() -> Self {
        Self {
            state: [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0],
            buffer: [0; 64],
            length: 0,
        }
    }
}

impl Sha1 {
    /// Create a new SHA-1 hasher
    pub fn new() -> Self {
        Self::default()
    }

    /// Compute the SHA-1 digest of the given data
    pub fn digest(data: impl AsRef<[u8]>) -> [u8; 20] {
        let mut h = Self::new();
        h.update(data);
        h.finalize_reset()
    }

    /// Update the hasher with new data
    pub fn update(&mut self, data: impl AsRef<[u8]>) {
        let mut data = data.as_ref();
        while !data.is_empty() {
            let buffer_len = self.length as usize % 64;
            let to_copy = std::cmp::min(64 - buffer_len, data.len());
            self.buffer[buffer_len..buffer_len + to_copy].copy_from_slice(&data[..to_copy]);
            self.length += to_copy as u64;
            data = &data[to_copy..];
            if self.length.is_multiple_of(64) {
                self.process_block();
            }
        }
    }

    /// Finalize the hash and return the digest
    pub fn finalize(mut self) -> [u8; 20] {
        self.finalize_reset()
    }

    /// Finalize the hash, reset the hasher, and return the digest
    pub fn finalize_reset(&mut self) -> [u8; 20] {
        let mut padding = [0u8; 64];
        padding[0] = 0x80;
        let length_bits = self.length * 8;
        let padding_len = if self.length % 64 < 56 {
            56 - self.length % 64
        } else {
            64 + 56 - self.length % 64
        };
        self.update(&padding[..padding_len as usize]);
        self.update(length_bits.to_be_bytes());
        let mut result = [0u8; 20];
        for (i, chunk) in result.chunks_mut(4).enumerate() {
            chunk.copy_from_slice(&self.state[i].to_be_bytes());
        }
        self.reset();
        result
    }

    fn reset(&mut self) {
        *self = Self::default();
    }

    #[allow(unsafe_code)]
    fn process_block(&mut self) {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "x86_64")] {
                if is_x86_feature_detected!("sha")
                    && is_x86_feature_detected!("ssse3")
                    && is_x86_feature_detected!("sse4.1")
                {
                    // SAFETY: Runtime detection confirmed every non-baseline CPU feature used by this code path.
                    unsafe { self.process_block_x86_sha() }
                } else {
                    self.process_block_software()
                }
            } else if #[cfg(target_arch = "aarch64")] {
                if std::arch::is_aarch64_feature_detected!("sha2") {
                    // SAFETY: is_aarch64_feature_detected! confirmed the sha2 hardware feature is available.
                    unsafe { self.process_block_aarch64_sha() }
                } else {
                    self.process_block_software()
                }
            } else {
                self.process_block_software()
            }
        }
    }

    // MARK: Software
    fn process_block_software(&mut self) {
        let mut w = [0u32; 80];
        for (i, chunk) in self.buffer.chunks(4).enumerate() {
            w[i] = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }

        let mut a = self.state[0];
        let mut b = self.state[1];
        let mut c = self.state[2];
        let mut d = self.state[3];
        let mut e = self.state[4];

        for (i, w_i) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | (!b & d), 0x5A827999u32),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                60..=79 => (b ^ c ^ d, 0xCA62C1D6),
                _ => unreachable!(),
            };

            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(*w_i);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
    }

    // MARK: x86_64 SHA-NI
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "sha,sse2,ssse3,sse4.1")]
    #[allow(unsafe_code)]
    unsafe fn process_block_x86_sha(&mut self) {
        // SAFETY: The caller verified sha/sse2/ssse3/sse4.1 feature availability; self.state and self.buffer are properly initialized.
        unsafe {
            use std::arch::x86_64::*;

            let mask = _mm_set_epi64x(0x0001_0203_0405_0607, 0x0809_0a0b_0c0d_0e0f_u64 as i64);

            let mut abcd = _mm_loadu_si128(self.state.as_ptr().cast());
            let mut e0 = _mm_set_epi32(self.state[4] as i32, 0, 0, 0);
            abcd = _mm_shuffle_epi32(abcd, 0x1B);
            let abcd_save = abcd;
            let e0_save = e0;

            let mut msg0 = _mm_shuffle_epi8(_mm_loadu_si128(self.buffer.as_ptr().cast()), mask);
            let mut msg1 =
                _mm_shuffle_epi8(_mm_loadu_si128(self.buffer.as_ptr().add(16).cast()), mask);
            let mut msg2 =
                _mm_shuffle_epi8(_mm_loadu_si128(self.buffer.as_ptr().add(32).cast()), mask);
            let mut msg3 =
                _mm_shuffle_epi8(_mm_loadu_si128(self.buffer.as_ptr().add(48).cast()), mask);

            let mut e1;

            // Rounds 0-3
            e0 = _mm_add_epi32(e0, msg0);
            e1 = abcd;
            abcd = _mm_sha1rnds4_epu32(abcd, e0, 0);

            // Rounds 4-7
            e1 = _mm_sha1nexte_epu32(e1, msg1);
            e0 = abcd;
            abcd = _mm_sha1rnds4_epu32(abcd, e1, 0);
            msg0 = _mm_sha1msg1_epu32(msg0, msg1);

            // Rounds 8-11
            e0 = _mm_sha1nexte_epu32(e0, msg2);
            e1 = abcd;
            abcd = _mm_sha1rnds4_epu32(abcd, e0, 0);
            msg1 = _mm_sha1msg1_epu32(msg1, msg2);
            msg0 = _mm_xor_si128(msg0, msg2);

            // Rounds 12-15
            e1 = _mm_sha1nexte_epu32(e1, msg3);
            e0 = abcd;
            msg0 = _mm_sha1msg2_epu32(msg0, msg3);
            abcd = _mm_sha1rnds4_epu32(abcd, e1, 0);
            msg2 = _mm_sha1msg1_epu32(msg2, msg3);
            msg1 = _mm_xor_si128(msg1, msg3);

            // Rounds 16-19
            e0 = _mm_sha1nexte_epu32(e0, msg0);
            e1 = abcd;
            msg1 = _mm_sha1msg2_epu32(msg1, msg0);
            abcd = _mm_sha1rnds4_epu32(abcd, e0, 0);
            msg3 = _mm_sha1msg1_epu32(msg3, msg0);
            msg2 = _mm_xor_si128(msg2, msg0);

            // Rounds 20-23
            e1 = _mm_sha1nexte_epu32(e1, msg1);
            e0 = abcd;
            msg2 = _mm_sha1msg2_epu32(msg2, msg1);
            abcd = _mm_sha1rnds4_epu32(abcd, e1, 1);
            msg0 = _mm_sha1msg1_epu32(msg0, msg1);
            msg3 = _mm_xor_si128(msg3, msg1);

            // Rounds 24-27
            e0 = _mm_sha1nexte_epu32(e0, msg2);
            e1 = abcd;
            msg3 = _mm_sha1msg2_epu32(msg3, msg2);
            abcd = _mm_sha1rnds4_epu32(abcd, e0, 1);
            msg1 = _mm_sha1msg1_epu32(msg1, msg2);
            msg0 = _mm_xor_si128(msg0, msg2);

            // Rounds 28-31
            e1 = _mm_sha1nexte_epu32(e1, msg3);
            e0 = abcd;
            msg0 = _mm_sha1msg2_epu32(msg0, msg3);
            abcd = _mm_sha1rnds4_epu32(abcd, e1, 1);
            msg2 = _mm_sha1msg1_epu32(msg2, msg3);
            msg1 = _mm_xor_si128(msg1, msg3);

            // Rounds 32-35
            e0 = _mm_sha1nexte_epu32(e0, msg0);
            e1 = abcd;
            msg1 = _mm_sha1msg2_epu32(msg1, msg0);
            abcd = _mm_sha1rnds4_epu32(abcd, e0, 1);
            msg3 = _mm_sha1msg1_epu32(msg3, msg0);
            msg2 = _mm_xor_si128(msg2, msg0);

            // Rounds 36-39
            e1 = _mm_sha1nexte_epu32(e1, msg1);
            e0 = abcd;
            msg2 = _mm_sha1msg2_epu32(msg2, msg1);
            abcd = _mm_sha1rnds4_epu32(abcd, e1, 1);
            msg0 = _mm_sha1msg1_epu32(msg0, msg1);
            msg3 = _mm_xor_si128(msg3, msg1);

            // Rounds 40-43
            e0 = _mm_sha1nexte_epu32(e0, msg2);
            e1 = abcd;
            msg3 = _mm_sha1msg2_epu32(msg3, msg2);
            abcd = _mm_sha1rnds4_epu32(abcd, e0, 2);
            msg1 = _mm_sha1msg1_epu32(msg1, msg2);
            msg0 = _mm_xor_si128(msg0, msg2);

            // Rounds 44-47
            e1 = _mm_sha1nexte_epu32(e1, msg3);
            e0 = abcd;
            msg0 = _mm_sha1msg2_epu32(msg0, msg3);
            abcd = _mm_sha1rnds4_epu32(abcd, e1, 2);
            msg2 = _mm_sha1msg1_epu32(msg2, msg3);
            msg1 = _mm_xor_si128(msg1, msg3);

            // Rounds 48-51
            e0 = _mm_sha1nexte_epu32(e0, msg0);
            e1 = abcd;
            msg1 = _mm_sha1msg2_epu32(msg1, msg0);
            abcd = _mm_sha1rnds4_epu32(abcd, e0, 2);
            msg3 = _mm_sha1msg1_epu32(msg3, msg0);
            msg2 = _mm_xor_si128(msg2, msg0);

            // Rounds 52-55
            e1 = _mm_sha1nexte_epu32(e1, msg1);
            e0 = abcd;
            msg2 = _mm_sha1msg2_epu32(msg2, msg1);
            abcd = _mm_sha1rnds4_epu32(abcd, e1, 2);
            msg0 = _mm_sha1msg1_epu32(msg0, msg1);
            msg3 = _mm_xor_si128(msg3, msg1);

            // Rounds 56-59
            e0 = _mm_sha1nexte_epu32(e0, msg2);
            e1 = abcd;
            msg3 = _mm_sha1msg2_epu32(msg3, msg2);
            abcd = _mm_sha1rnds4_epu32(abcd, e0, 2);
            msg1 = _mm_sha1msg1_epu32(msg1, msg2);
            msg0 = _mm_xor_si128(msg0, msg2);

            // Rounds 60-63
            e1 = _mm_sha1nexte_epu32(e1, msg3);
            e0 = abcd;
            msg0 = _mm_sha1msg2_epu32(msg0, msg3);
            abcd = _mm_sha1rnds4_epu32(abcd, e1, 3);
            msg2 = _mm_sha1msg1_epu32(msg2, msg3);
            msg1 = _mm_xor_si128(msg1, msg3);

            // Rounds 64-67
            e0 = _mm_sha1nexte_epu32(e0, msg0);
            e1 = abcd;
            msg1 = _mm_sha1msg2_epu32(msg1, msg0);
            abcd = _mm_sha1rnds4_epu32(abcd, e0, 3);
            msg3 = _mm_sha1msg1_epu32(msg3, msg0);
            msg2 = _mm_xor_si128(msg2, msg0);

            // Rounds 68-71
            e1 = _mm_sha1nexte_epu32(e1, msg1);
            e0 = abcd;
            msg2 = _mm_sha1msg2_epu32(msg2, msg1);
            abcd = _mm_sha1rnds4_epu32(abcd, e1, 3);
            msg3 = _mm_xor_si128(msg3, msg1);

            // Rounds 72-75
            e0 = _mm_sha1nexte_epu32(e0, msg2);
            e1 = abcd;
            msg3 = _mm_sha1msg2_epu32(msg3, msg2);
            abcd = _mm_sha1rnds4_epu32(abcd, e0, 3);

            // Rounds 76-79
            e1 = _mm_sha1nexte_epu32(e1, msg3);
            e0 = abcd;
            abcd = _mm_sha1rnds4_epu32(abcd, e1, 3);

            // Combine state
            e0 = _mm_sha1nexte_epu32(e0, e0_save);
            abcd = _mm_add_epi32(abcd, abcd_save);

            // Store state
            abcd = _mm_shuffle_epi32(abcd, 0x1B);
            _mm_storeu_si128(self.state.as_mut_ptr().cast(), abcd);
            self.state[4] = _mm_extract_epi32::<3>(e0) as u32;
        }
    }

    // MARK: aarch64 SHA
    #[cfg(target_arch = "aarch64")]
    #[target_feature(enable = "sha2")]
    #[allow(unsafe_code)]
    unsafe fn process_block_aarch64_sha(&mut self) {
        // SAFETY: The caller verified sha2 feature availability; self.state and self.buffer are properly initialized.
        unsafe {
            use std::arch::aarch64::*;

            // Load state
            let mut abcd = vld1q_u32(self.state.as_ptr());
            let mut e = self.state[4];
            let abcd_save = abcd;
            let e_save = e;

            // Load and byte-swap message words
            let mut msg0 = vreinterpretq_u32_u8(vrev32q_u8(vreinterpretq_u8_u32(vld1q_u32(
                self.buffer.as_ptr().cast(),
            ))));
            let mut msg1 = vreinterpretq_u32_u8(vrev32q_u8(vreinterpretq_u8_u32(vld1q_u32(
                self.buffer.as_ptr().add(16).cast(),
            ))));
            let mut msg2 = vreinterpretq_u32_u8(vrev32q_u8(vreinterpretq_u8_u32(vld1q_u32(
                self.buffer.as_ptr().add(32).cast(),
            ))));
            let mut msg3 = vreinterpretq_u32_u8(vrev32q_u8(vreinterpretq_u8_u32(vld1q_u32(
                self.buffer.as_ptr().add(48).cast(),
            ))));

            let mut tmp0;
            let mut tmp1;

            // Rounds 0-3
            tmp0 = vaddq_u32(msg0, vdupq_n_u32(0x5A827999));
            tmp1 = abcd;
            abcd = vsha1cq_u32(abcd, e, tmp0);
            e = vsha1h_u32(vgetq_lane_u32(tmp1, 0));
            tmp0 = vsha1su0q_u32(msg0, msg1, msg2);
            msg0 = vsha1su1q_u32(tmp0, msg3);

            // Rounds 4-7
            tmp0 = vaddq_u32(msg1, vdupq_n_u32(0x5A827999));
            tmp1 = abcd;
            abcd = vsha1cq_u32(abcd, e, tmp0);
            e = vsha1h_u32(vgetq_lane_u32(tmp1, 0));
            tmp0 = vsha1su0q_u32(msg1, msg2, msg3);
            msg1 = vsha1su1q_u32(tmp0, msg0);

            // Rounds 8-11
            tmp0 = vaddq_u32(msg2, vdupq_n_u32(0x5A827999));
            tmp1 = abcd;
            abcd = vsha1cq_u32(abcd, e, tmp0);
            e = vsha1h_u32(vgetq_lane_u32(tmp1, 0));
            tmp0 = vsha1su0q_u32(msg2, msg3, msg0);
            msg2 = vsha1su1q_u32(tmp0, msg1);

            // Rounds 12-15
            tmp0 = vaddq_u32(msg3, vdupq_n_u32(0x5A827999));
            tmp1 = abcd;
            abcd = vsha1cq_u32(abcd, e, tmp0);
            e = vsha1h_u32(vgetq_lane_u32(tmp1, 0));
            tmp0 = vsha1su0q_u32(msg3, msg0, msg1);
            msg3 = vsha1su1q_u32(tmp0, msg2);

            // Rounds 16-19
            tmp0 = vaddq_u32(msg0, vdupq_n_u32(0x5A827999));
            tmp1 = abcd;
            abcd = vsha1cq_u32(abcd, e, tmp0);
            e = vsha1h_u32(vgetq_lane_u32(tmp1, 0));
            tmp0 = vsha1su0q_u32(msg0, msg1, msg2);
            msg0 = vsha1su1q_u32(tmp0, msg3);

            // Rounds 20-23
            tmp0 = vaddq_u32(msg1, vdupq_n_u32(0x6ED9EBA1));
            tmp1 = abcd;
            abcd = vsha1pq_u32(abcd, e, tmp0);
            e = vsha1h_u32(vgetq_lane_u32(tmp1, 0));
            tmp0 = vsha1su0q_u32(msg1, msg2, msg3);
            msg1 = vsha1su1q_u32(tmp0, msg0);

            // Rounds 24-27
            tmp0 = vaddq_u32(msg2, vdupq_n_u32(0x6ED9EBA1));
            tmp1 = abcd;
            abcd = vsha1pq_u32(abcd, e, tmp0);
            e = vsha1h_u32(vgetq_lane_u32(tmp1, 0));
            tmp0 = vsha1su0q_u32(msg2, msg3, msg0);
            msg2 = vsha1su1q_u32(tmp0, msg1);

            // Rounds 28-31
            tmp0 = vaddq_u32(msg3, vdupq_n_u32(0x6ED9EBA1));
            tmp1 = abcd;
            abcd = vsha1pq_u32(abcd, e, tmp0);
            e = vsha1h_u32(vgetq_lane_u32(tmp1, 0));
            tmp0 = vsha1su0q_u32(msg3, msg0, msg1);
            msg3 = vsha1su1q_u32(tmp0, msg2);

            // Rounds 32-35
            tmp0 = vaddq_u32(msg0, vdupq_n_u32(0x6ED9EBA1));
            tmp1 = abcd;
            abcd = vsha1pq_u32(abcd, e, tmp0);
            e = vsha1h_u32(vgetq_lane_u32(tmp1, 0));
            tmp0 = vsha1su0q_u32(msg0, msg1, msg2);
            msg0 = vsha1su1q_u32(tmp0, msg3);

            // Rounds 36-39
            tmp0 = vaddq_u32(msg1, vdupq_n_u32(0x6ED9EBA1));
            tmp1 = abcd;
            abcd = vsha1pq_u32(abcd, e, tmp0);
            e = vsha1h_u32(vgetq_lane_u32(tmp1, 0));
            tmp0 = vsha1su0q_u32(msg1, msg2, msg3);
            msg1 = vsha1su1q_u32(tmp0, msg0);

            // Rounds 40-43
            tmp0 = vaddq_u32(msg2, vdupq_n_u32(0x8F1BBCDC));
            tmp1 = abcd;
            abcd = vsha1mq_u32(abcd, e, tmp0);
            e = vsha1h_u32(vgetq_lane_u32(tmp1, 0));
            tmp0 = vsha1su0q_u32(msg2, msg3, msg0);
            msg2 = vsha1su1q_u32(tmp0, msg1);

            // Rounds 44-47
            tmp0 = vaddq_u32(msg3, vdupq_n_u32(0x8F1BBCDC));
            tmp1 = abcd;
            abcd = vsha1mq_u32(abcd, e, tmp0);
            e = vsha1h_u32(vgetq_lane_u32(tmp1, 0));
            tmp0 = vsha1su0q_u32(msg3, msg0, msg1);
            msg3 = vsha1su1q_u32(tmp0, msg2);

            // Rounds 48-51
            tmp0 = vaddq_u32(msg0, vdupq_n_u32(0x8F1BBCDC));
            tmp1 = abcd;
            abcd = vsha1mq_u32(abcd, e, tmp0);
            e = vsha1h_u32(vgetq_lane_u32(tmp1, 0));
            tmp0 = vsha1su0q_u32(msg0, msg1, msg2);
            msg0 = vsha1su1q_u32(tmp0, msg3);

            // Rounds 52-55
            tmp0 = vaddq_u32(msg1, vdupq_n_u32(0x8F1BBCDC));
            tmp1 = abcd;
            abcd = vsha1mq_u32(abcd, e, tmp0);
            e = vsha1h_u32(vgetq_lane_u32(tmp1, 0));
            tmp0 = vsha1su0q_u32(msg1, msg2, msg3);
            msg1 = vsha1su1q_u32(tmp0, msg0);

            // Rounds 56-59
            tmp0 = vaddq_u32(msg2, vdupq_n_u32(0x8F1BBCDC));
            tmp1 = abcd;
            abcd = vsha1mq_u32(abcd, e, tmp0);
            e = vsha1h_u32(vgetq_lane_u32(tmp1, 0));
            tmp0 = vsha1su0q_u32(msg2, msg3, msg0);
            msg2 = vsha1su1q_u32(tmp0, msg1);

            // Rounds 60-63
            tmp0 = vaddq_u32(msg3, vdupq_n_u32(0xCA62C1D6));
            tmp1 = abcd;
            abcd = vsha1pq_u32(abcd, e, tmp0);
            e = vsha1h_u32(vgetq_lane_u32(tmp1, 0));
            tmp0 = vsha1su0q_u32(msg3, msg0, msg1);
            msg3 = vsha1su1q_u32(tmp0, msg2);

            // Rounds 64-67
            tmp0 = vaddq_u32(msg0, vdupq_n_u32(0xCA62C1D6));
            tmp1 = abcd;
            abcd = vsha1pq_u32(abcd, e, tmp0);
            e = vsha1h_u32(vgetq_lane_u32(tmp1, 0));
            tmp0 = vsha1su0q_u32(msg0, msg1, msg2);
            msg0 = vsha1su1q_u32(tmp0, msg3);

            // Rounds 68-71
            tmp0 = vaddq_u32(msg1, vdupq_n_u32(0xCA62C1D6));
            tmp1 = abcd;
            abcd = vsha1pq_u32(abcd, e, tmp0);
            e = vsha1h_u32(vgetq_lane_u32(tmp1, 0));
            tmp0 = vsha1su0q_u32(msg1, msg2, msg3);
            let _ = vsha1su1q_u32(tmp0, msg0);

            // Rounds 72-75
            tmp0 = vaddq_u32(msg2, vdupq_n_u32(0xCA62C1D6));
            tmp1 = abcd;
            abcd = vsha1pq_u32(abcd, e, tmp0);
            e = vsha1h_u32(vgetq_lane_u32(tmp1, 0));

            // Rounds 76-79
            tmp0 = vaddq_u32(msg3, vdupq_n_u32(0xCA62C1D6));
            tmp1 = abcd;
            abcd = vsha1pq_u32(abcd, e, tmp0);
            e = vsha1h_u32(vgetq_lane_u32(tmp1, 0));

            // Add saved state
            abcd = vaddq_u32(abcd, abcd_save);
            e = e.wrapping_add(e_save);

            // Store state
            vst1q_u32(self.state.as_mut_ptr(), abcd);
            self.state[4] = e;
        }
    }
}

// MARK: Digest impl
impl digest::Digest for Sha1 {
    const BLOCK_SIZE: usize = 64;
    type Output = [u8; 20];

    fn update(&mut self, data: &[u8]) {
        self.update(data);
    }

    fn finalize_reset(&mut self) -> Self::Output {
        self.finalize_reset()
    }
}

// MARK: Tests
#[cfg(test)]
mod test {
    use super::*;

    fn to_hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    #[test]
    fn test_sha1() {
        let test_cases = [
            ("", "da39a3ee5e6b4b0d3255bfef95601890afd80709"),
            ("abc", "a9993e364706816aba3e25717850c26c9cd0d89d"),
            ("hello world", "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed"),
            (
                "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
                "84983e441c3bd26ebaae4aa1f95129e5e54670f1",
            ),
        ];

        for (input, expected) in test_cases {
            let hash = Sha1::digest(input.as_bytes());
            assert_eq!(to_hex(&hash), expected, "Failed for input: {input}");
        }
    }

    #[test]
    fn test_sha1_million_a() {
        let mut hasher = Sha1::new();
        for _ in 0..10000 {
            hasher.update([b'a'; 100]);
        }
        assert_eq!(
            to_hex(&hasher.finalize()),
            "34aa973cd4c4daa4f61eeb2bdbad27316534016f"
        );
    }

    #[test]
    fn test_sha1_reset() {
        let mut hasher = Sha1::new();
        hasher.update(b"abc");
        hasher.finalize_reset();
        hasher.update(b"hello world");
        assert_eq!(
            to_hex(&hasher.finalize_reset()),
            "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed"
        );
    }
}
