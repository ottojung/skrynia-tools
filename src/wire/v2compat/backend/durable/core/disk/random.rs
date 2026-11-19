use skrynia_program0::cap::Random;
use std::fmt;

pub struct DiskRandom<R>(pub R);

#[derive(Debug)]
pub struct RandomError(String);

impl fmt::Display for RandomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl<R: Random> super::super::super::data::RandomSource for DiskRandom<R>
where
    R::Err: fmt::Display,
{
    type Error = RandomError;
    fn fill(&self, out: &mut [u8]) -> Result<(), Self::Error> {
        self.0.fill(out).map_err(|e| RandomError(e.to_string()))
    }
}
