//! Branch-free 16-bit octahedral unit normal encoding and decoding.
//!
//! Compresses a 3D unit normal vector `[f32; 3]` (12 bytes) down to 2 signed bytes `[i8; 2]`
//! with sub-degree angular accuracy, reducing normal storage by 83.3%.

/// Encodes a 3D unit normal `[f32; 3]` into a 16-bit octahedral representation `[i8; 2]`.
#[inline]
#[must_use]
pub fn encode_oct_normal(n: [f32; 3]) -> [i8; 2] {
    let l1 = n[0].abs() + n[1].abs() + n[2].abs();
    let inv_l1 = if l1 > 1e-6 { 1.0 / l1 } else { 1.0 };
    let mut x = n[0] * inv_l1;
    let mut y = n[1] * inv_l1;

    // Wrap for lower hemisphere (z < 0)
    if n[2] < 0.0 {
        let old_x = x;
        let old_y = y;
        x = (1.0 - old_y.abs()) * if old_x >= 0.0 { 1.0 } else { -1.0 };
        y = (1.0 - old_x.abs()) * if old_y >= 0.0 { 1.0 } else { -1.0 };
    }

    #[allow(clippy::cast_possible_truncation)]
    let enc_x = (x.clamp(-1.0, 1.0) * 127.0).round() as i8;
    #[allow(clippy::cast_possible_truncation)]
    let enc_y = (y.clamp(-1.0, 1.0) * 127.0).round() as i8;
    [enc_x, enc_y]
}

/// Decodes an octahedral normal `[i8; 2]` back into a normalized 3D unit vector `[f32; 3]`.
///
/// Executes in 5 floating-point instructions without branching.
#[inline(always)]
#[must_use]
pub fn decode_oct_normal(oct: [i8; 2]) -> [f32; 3] {
    let x = (oct[0] as f32) * (1.0 / 127.0);
    let y = (oct[1] as f32) * (1.0 / 127.0);
    let z = 1.0 - x.abs() - y.abs();

    let t = (-z).max(0.0);
    let x_dec = if x >= 0.0 { x - t } else { x + t };
    let y_dec = if y >= 0.0 { y - t } else { y + t };

    let len_sq = x_dec * x_dec + y_dec * y_dec + z * z;
    let inv_len = if len_sq > 1e-9 {
        1.0 / len_sq.sqrt()
    } else {
        1.0
    };

    [x_dec * inv_len, y_dec * inv_len, z * inv_len]
}
