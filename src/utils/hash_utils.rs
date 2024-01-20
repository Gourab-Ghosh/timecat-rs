use super::*;

pub trait CustomHash {
    fn hash(&self) -> u64;
}

impl CustomHash for SubBoard {
    #[inline(always)]
    fn hash(&self) -> u64 {
        self.get_hash().max(1)
    }
}
