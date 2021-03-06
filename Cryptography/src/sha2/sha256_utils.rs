use fake_simd::u32x4;
use crate::sha2::consts::{BLOCK_LEN, K32X4};
use block_buffer::byteorder::{BE, ByteOrder};

#[inline]
fn sha256load(v2: u32x4, v3: u32x4) -> u32x4 {
    u32x4(v3.3, v2.0, v2.1, v2.2)
}

#[inline]
fn sha256swap(v0: u32x4) -> u32x4 {
    u32x4(v0.2, v0.3, v0.0, v0.1)
}

fn sha256msg1(v0: u32x4, v1: u32x4) -> u32x4 {

    #[inline]
    fn sigma0x4(x: u32x4) -> u32x4 {
        ((x >> u32x4( 7,  7,  7,  7)) | (x << u32x4(25, 25, 25, 25))) ^
        ((x >> u32x4(18, 18, 18, 18)) | (x << u32x4(14, 14, 14, 14))) ^
         (x >> u32x4( 3,  3,  3,  3))
    }

    v0 + sigma0x4(sha256load(v0, v1))
}

fn sha256msg2(v4: u32x4, v3: u32x4) -> u32x4 {

    macro_rules! sigma1 {
        ($a:expr) => ($a.rotate_right(17) ^ $a.rotate_right(19) ^ ($a >> 10))
    }

    let u32x4(x3, x2, x1, x0) = v4;
    let u32x4(w15, w14, _, _) = v3;

    let w16 = x0.wrapping_add(sigma1!(w14));
    let w17 = x1.wrapping_add(sigma1!(w15));
    let w18 = x2.wrapping_add(sigma1!(w16));
    let w19 = x3.wrapping_add(sigma1!(w17));

    u32x4(w19, w18, w17, w16)
}


fn sha256_digest_round_x2(cdgh: u32x4, abef: u32x4, wk: u32x4) -> u32x4 {

    macro_rules! big_sigma0 {
        ($a:expr) => (($a.rotate_right(2) ^ $a.rotate_right(13) ^ $a.rotate_right(22)))
    }
    macro_rules! big_sigma1 {
        ($a:expr) => (($a.rotate_right(6) ^ $a.rotate_right(11) ^ $a.rotate_right(25)))
    }
    macro_rules! bool3ary_202 {
        ($a:expr, $b:expr, $c:expr) => ($c ^ ($a & ($b ^ $c)))
    } // Choose, MD5F, SHA1C
    macro_rules! bool3ary_232 {
        ($a:expr, $b:expr, $c:expr) => (($a & $b) ^ ($a & $c) ^ ($b & $c))
    } // Majority, SHA1M

    let u32x4(_, _, wk1, wk0) = wk;
    let u32x4(a0, b0, e0, f0) = abef;
    let u32x4(c0, d0, g0, h0) = cdgh;

    // a round
    let x0 = big_sigma1!(e0)
        .wrapping_add(bool3ary_202!(e0, f0, g0))
        .wrapping_add(wk0)
        .wrapping_add(h0);
    let y0 = big_sigma0!(a0).wrapping_add(bool3ary_232!(a0, b0, c0));
    let (a1, b1, c1, d1, e1, f1, g1, h1) =
        (x0.wrapping_add(y0), a0, b0, c0, x0.wrapping_add(d0), e0, f0, g0);

    // a round
    let x1 = big_sigma1!(e1)
        .wrapping_add(bool3ary_202!(e1, f1, g1))
        .wrapping_add(wk1)
        .wrapping_add(h1);
    let y1 = big_sigma0!(a1).wrapping_add(bool3ary_232!(a1, b1, c1));
    let (a2, b2, _, _, e2, f2, _, _) =
        (x1.wrapping_add(y1), a1, b1, c1, x1.wrapping_add(d1), e1, f1, g1);

    u32x4(a2, b2, e2, f2)
}

fn sha256_digest_block_u32(state: &mut [u32; 8], block: &[u32; 16]) {
    let k = &K32X4;

    macro_rules! schedule {
        ($v0:expr, $v1:expr, $v2:expr, $v3:expr) => (
            sha256msg2(sha256msg1($v0, $v1) + sha256load($v2, $v3), $v3)
        )
    }

    macro_rules! rounds4 {
        ($abef:ident, $cdgh:ident, $rest:expr) => {
            {
                $cdgh = sha256_digest_round_x2($cdgh, $abef, $rest);
                $abef = sha256_digest_round_x2($abef, $cdgh, sha256swap($rest));
            }
        }
    }

    let mut abef = u32x4(state[0], state[1], state[4], state[5]);
    let mut cdgh = u32x4(state[2], state[3], state[6], state[7]);

    // Rounds 0..64
    let mut w0 = u32x4(block[3], block[2], block[1], block[0]);
    rounds4!(abef, cdgh, k[0] + w0);
    let mut w1 = u32x4(block[7], block[6], block[5], block[4]);
    rounds4!(abef, cdgh, k[1] + w1);
    let mut w2 = u32x4(block[11], block[10], block[9], block[8]);
    rounds4!(abef, cdgh, k[2] + w2);
    let mut w3 = u32x4(block[15], block[14], block[13], block[12]);
    rounds4!(abef, cdgh, k[3] + w3);
    let mut w4 = schedule!(w0, w1, w2, w3);
    rounds4!(abef, cdgh, k[4] + w4);
    w0 = schedule!(w1, w2, w3, w4);
    rounds4!(abef, cdgh, k[5] + w0);
    w1 = schedule!(w2, w3, w4, w0);
    rounds4!(abef, cdgh, k[6] + w1);
    w2 = schedule!(w3, w4, w0, w1);
    rounds4!(abef, cdgh, k[7] + w2);
    w3 = schedule!(w4, w0, w1, w2);
    rounds4!(abef, cdgh, k[8] + w3);
    w4 = schedule!(w0, w1, w2, w3);
    rounds4!(abef, cdgh, k[9] + w4);
    w0 = schedule!(w1, w2, w3, w4);
    rounds4!(abef, cdgh, k[10] + w0);
    w1 = schedule!(w2, w3, w4, w0);
    rounds4!(abef, cdgh, k[11] + w1);
    w2 = schedule!(w3, w4, w0, w1);
    rounds4!(abef, cdgh, k[12] + w2);
    w3 = schedule!(w4, w0, w1, w2);
    rounds4!(abef, cdgh, k[13] + w3);
    w4 = schedule!(w0, w1, w2, w3);
    rounds4!(abef, cdgh, k[14] + w4);
    w0 = schedule!(w1, w2, w3, w4);
    rounds4!(abef, cdgh, k[15] + w0);

    let u32x4(a, b, e, f) = abef;
    let u32x4(c, d, g, h) = cdgh;

    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
    state[5] = state[5].wrapping_add(f);
    state[6] = state[6].wrapping_add(g);
    state[7] = state[7].wrapping_add(h);
}

pub fn compress256(state: &mut [u32; 8], block: &[u8; 64]) {
    let mut block_u32 = [0u32; BLOCK_LEN];
    BE::read_u32_into(block, &mut block_u32[..]);
    sha256_digest_block_u32(state, &block_u32);
}
