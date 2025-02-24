pub const U8_BYTES: usize = size_of::<u8>();
pub const U16_BYTES: usize = size_of::<u16>();
pub const U32_BYTES: usize = size_of::<u32>();
pub const U64_BYTES: usize = size_of::<u64>();

#[macro_export]
macro_rules! read {
    ($name:ident, $type:ty, $size:expr) => {
        fn $name(&mut self) -> Result<$type> {
            self.eof($size)?;

            let slice = &self.data[self.cursor..self.cursor + $size];
            let b = <$type>::from_be_bytes(slice.try_into()?);
            self.cursor += $size;

            Ok(b)
        }
    };
}

#[macro_export]
macro_rules! eof {
    () => {
        fn eof(&self, len: usize) -> Result<()> {
            let end = self.data.len();

            ensure!(
                self.cursor + len.saturating_sub(1) < self.data.len(),
                "Unexpected EOF. At {}, seek by {}, buffer size: {}.",
                self.cursor,
                len,
                end
            );

            Ok(())
        }
    };
}
