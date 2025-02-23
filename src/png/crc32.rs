#[cfg(all(target_arch = "aarch64", target_feature = "crc"))]
pub fn compute_crc<'a>(chunk_type: &'a [u8], chunk_data: &'a [u8]) -> u32 {
    use std::arch::aarch64::__crc32b;

    let mut crc = 0xffff_ffff;

    for &byte in chunk_type {
        crc = unsafe { __crc32b(crc, byte) };
    }

    for &byte in chunk_data {
        crc = unsafe { __crc32b(crc, byte) };
    }

    !crc
}

#[cfg(not(all(target_arch = "aarch64", target_feature = "crc")))]
pub fn compute_crc<'a>(chunk_type: &'a [u8], chunk_data: &'a [u8]) -> u32 {
    use crc32fast::Hasher;

    let mut hx = Hasher::new();
    hx.update(chunk_type);
    hx.update(chunk_data);

    hx.finalize()
}
