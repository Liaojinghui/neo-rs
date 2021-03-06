use fake_simd::u64x2;
use crate::sha2::consts::{BLOCK_LEN, K64X2};
use block_buffer::byteorder::{BE, ByteOrder};

#[inline]
fn sha512load(v0: u64x2, v1: u64x2) -> u64x2 {
    u64x2(v1.1, v0.0)
}

pub fn sha512_schedule_x2(v0: u64x2, v1: u64x2, v4to5: u64x2, v7: u64x2)
                          -> u64x2 {

    fn sigma0(x: u64) -> u64 {
        ((x << 63) | (x >> 1)) ^ ((x << 56) | (x >> 8)) ^ (x >> 7)
    }

    fn sigma1(x: u64) -> u64 {
        ((x << 45) | (x >> 19)) ^ ((x << 3) | (x >> 61)) ^ (x >> 6)
    }

    let u64x2(w1, w0) = v0;
    let u64x2(_, w2) = v1;
    let u64x2(w10, w9) = v4to5;
    let u64x2(w15, w14) = v7;

    let w16 =
        sigma1(w14).wrapping_add(w9).wrapping_add(sigma0(w1)).wrapping_add(w0);
    let w17 =
        sigma1(w15).wrapping_add(w10).wrapping_add(sigma0(w2)).wrapping_add(w1);

    u64x2(w17, w16)
}

pub fn sha512_digest_round(ae: u64x2, bf: u64x2, cg: u64x2, dh: u64x2,
                           wk0: u64)
                           -> u64x2 {

    macro_rules! big_sigma0 {
        ($a:expr) => (($a.rotate_right(28) ^ $a.rotate_right(34) ^ $a.rotate_right(39)))
    }
    macro_rules! big_sigma1 {
        ($a:expr) => (($a.rotate_right(14) ^ $a.rotate_right(18) ^ $a.rotate_right(41)))
    }
    macro_rules! bool3ary_202 {
        ($a:expr, $b:expr, $c:expr) => ($c ^ ($a & ($b ^ $c)))
    } // Choose, MD5F, SHA1C
    macro_rules! bool3ary_232 {
        ($a:expr, $b:expr, $c:expr) => (($a & $b) ^ ($a & $c) ^ ($b & $c))
    } // Majority, SHA1M

    let u64x2(a0, e0) = ae;
    let u64x2(b0, f0) = bf;
    let u64x2(c0, g0) = cg;
    let u64x2(d0, h0) = dh;

    // a round
    let x0 = big_sigma1!(e0)
        .wrapping_add(bool3ary_202!(e0, f0, g0))
        .wrapping_add(wk0)
        .wrapping_add(h0);
    let y0 = big_sigma0!(a0).wrapping_add(bool3ary_232!(a0, b0, c0));
    let (a1, _, _, _, e1, _, _, _) =
        (x0.wrapping_add(y0), a0, b0, c0, x0.wrapping_add(d0), e0, f0, g0);

    u64x2(a1, e1)
}

pub fn sha512_digest_block_u64(state: &mut [u64; 8], block: &[u64; 16]) {
    let k = &K64X2;

    macro_rules! schedule {
        ($v0:expr, $v1:expr, $v4:expr, $v5:expr, $v7:expr) => (
             sha512_schedule_x2($v0, $v1, sha512load($v4, $v5), $v7)
        )
    }

    macro_rules! rounds4 {
        ($ae:ident, $bf:ident, $cg:ident, $dh:ident, $wk0:expr, $wk1:expr) => {
            {
                let u64x2(u, t) = $wk0;
                let u64x2(w, v) = $wk1;

                $dh = sha512_digest_round($ae, $bf, $cg, $dh, t);
                $cg = sha512_digest_round($dh, $ae, $bf, $cg, u);
                $bf = sha512_digest_round($cg, $dh, $ae, $bf, v);
                $ae = sha512_digest_round($bf, $cg, $dh, $ae, w);
            }
        }
    }

    let mut ae = u64x2(state[0], state[4]);
    let mut bf = u64x2(state[1], state[5]);
    let mut cg = u64x2(state[2], state[6]);
    let mut dh = u64x2(state[3], state[7]);

    let (mut w1, mut w0) = (u64x2(block[3], block[2]),
                            u64x2(block[1], block[0]));
    rounds4!(ae, bf, cg, dh, k[0] + w0, k[1] + w1);
    let (mut w3, mut w2) = (u64x2(block[7], block[6]),
                            u64x2(block[5], block[4]));
    rounds4!(ae, bf, cg, dh, k[2] + w2, k[3] + w3);
    let (mut w5, mut w4) = (u64x2(block[11], block[10]),
                            u64x2(block[9], block[8]));
    rounds4!(ae, bf, cg, dh, k[4] + w4, k[5] + w5);
    let (mut w7, mut w6) = (u64x2(block[15], block[14]),
                            u64x2(block[13], block[12]));
    rounds4!(ae, bf, cg, dh, k[6] + w6, k[7] + w7);
    let mut w8 = schedule!(w0, w1, w4, w5, w7);
    let mut w9 = schedule!(w1, w2, w5, w6, w8);
    rounds4!(ae, bf, cg, dh, k[8] + w8, k[9] + w9);

    // Rounds 20..40
    w0 = schedule!(w2, w3, w6, w7, w9);
    w1 = schedule!(w3, w4, w7, w8, w0);
    rounds4!(ae, bf, cg, dh, k[10] + w0, k[11] + w1);
    w2 = schedule!(w4, w5, w8, w9, w1);
    w3 = schedule!(w5, w6, w9, w0, w2);
    rounds4!(ae, bf, cg, dh, k[12] + w2, k[13] + w3);
    w4 = schedule!(w6, w7, w0, w1, w3);
    w5 = schedule!(w7, w8, w1, w2, w4);
    rounds4!(ae, bf, cg, dh, k[14] + w4, k[15] + w5);
    w6 = schedule!(w8, w9, w2, w3, w5);
    w7 = schedule!(w9, w0, w3, w4, w6);
    rounds4!(ae, bf, cg, dh, k[16] + w6, k[17] + w7);
    w8 = schedule!(w0, w1, w4, w5, w7);
    w9 = schedule!(w1, w2, w5, w6, w8);
    rounds4!(ae, bf, cg, dh, k[18] + w8, k[19] + w9);

    // Rounds 40..60
    w0 = schedule!(w2, w3, w6, w7, w9);
    w1 = schedule!(w3, w4, w7, w8, w0);
    rounds4!(ae, bf, cg, dh, k[20] + w0, k[21] + w1);
    w2 = schedule!(w4, w5, w8, w9, w1);
    w3 = schedule!(w5, w6, w9, w0, w2);
    rounds4!(ae, bf, cg, dh, k[22] + w2, k[23] + w3);
    w4 = schedule!(w6, w7, w0, w1, w3);
    w5 = schedule!(w7, w8, w1, w2, w4);
    rounds4!(ae, bf, cg, dh, k[24] + w4, k[25] + w5);
    w6 = schedule!(w8, w9, w2, w3, w5);
    w7 = schedule!(w9, w0, w3, w4, w6);
    rounds4!(ae, bf, cg, dh, k[26] + w6, k[27] + w7);
    w8 = schedule!(w0, w1, w4, w5, w7);
    w9 = schedule!(w1, w2, w5, w6, w8);
    rounds4!(ae, bf, cg, dh, k[28] + w8, k[29] + w9);

    // Rounds 60..80
    w0 = schedule!(w2, w3, w6, w7, w9);
    w1 = schedule!(w3, w4, w7, w8, w0);
    rounds4!(ae, bf, cg, dh, k[30] + w0, k[31] + w1);
    w2 = schedule!(w4, w5, w8, w9, w1);
    w3 = schedule!(w5, w6, w9, w0, w2);
    rounds4!(ae, bf, cg, dh, k[32] + w2, k[33] + w3);
    w4 = schedule!(w6, w7, w0, w1, w3);
    w5 = schedule!(w7, w8, w1, w2, w4);
    rounds4!(ae, bf, cg, dh, k[34] + w4, k[35] + w5);
    w6 = schedule!(w8, w9, w2, w3, w5);
    w7 = schedule!(w9, w0, w3, w4, w6);
    rounds4!(ae, bf, cg, dh, k[36] + w6, k[37] + w7);
    w8 = schedule!(w0, w1, w4, w5, w7);
    w9 = schedule!(w1, w2, w5, w6, w8);
    rounds4!(ae, bf, cg, dh, k[38] + w8, k[39] + w9);

    let u64x2(a, e) = ae;
    let u64x2(b, f) = bf;
    let u64x2(c, g) = cg;
    let u64x2(d, h) = dh;

    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
    state[5] = state[5].wrapping_add(f);
    state[6] = state[6].wrapping_add(g);
    state[7] = state[7].wrapping_add(h);
}

pub fn compress512(state: &mut [u64; 8], block: &[u8; 128]) {
    let mut block_u64 = [0u64; BLOCK_LEN];
    BE::read_u64_into(block, &mut block_u64[..]);
    sha512_digest_block_u64(state, &block_u64);
}