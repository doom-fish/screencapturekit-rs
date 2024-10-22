use std::hash::{self, DefaultHasher, Hasher};
pub fn hash<T: hash::Hash>(t: T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
