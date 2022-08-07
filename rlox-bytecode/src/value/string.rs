use std::{
    hash::{BuildHasher, Hash, Hasher},
    marker::PhantomData,
    ops::Deref,
};

use crate::table::FNV1aBuilder;

#[derive(PartialEq, Eq)]
pub struct HashedString<S: BuildHasher = FNV1aBuilder> {
    hash: u32,
    inner: String,
    _marker: PhantomData<S>,
}

impl<B: BuildHasher + Default> From<String> for HashedString<B> {
    fn from(inner: String) -> Self {
        let _builder = B::default();
        let mut hasher = _builder.build_hasher();
        inner.hash(&mut hasher);
        let hash = hasher.finish() as u32;
        Self {
            inner,
            hash,
            _marker: PhantomData::default(),
        }
    }
}

impl Hash for HashedString {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl Deref for HashedString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<B: BuildHasher + Default> PartialEq<str> for HashedString<B> {
    fn eq(&self, other: &str) -> bool {
        let mut hasher = B::default().build_hasher();
        other.hash(&mut hasher);
        self.hash == (hasher.finish() as u32) && self.inner == other
    }
}
