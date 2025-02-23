use afl::fuzz;
use her::PngDecoder;

fn main() {
    fuzz!(|data: &[u8]| {
        let mut decoder = PngDecoder::new(data);
        let _ = decoder.decode();
    });
}
