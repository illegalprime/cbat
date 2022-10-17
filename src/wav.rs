/// helper for working with wave files
pub struct Wav16<'a>(&'a [u8]);

impl<'a> Wav16<'a> {
    /// wrap raw wave file data in this object
    pub fn new(data: &'a [u8]) -> Self {
        Self(&data[44..]) // skip the header
    }

    /// iterates over every sample and converts it to 32-bit depth
    pub fn stream32(&'a self) -> impl Iterator<Item = u32> + 'a {
        (0..self.0.len()).step_by(2).map(|i| {
            let word = i16::from_le_bytes([self.0[i], self.0[i + 1]]);
            ((word as i32) << 16) as u32
        })
    }
}
