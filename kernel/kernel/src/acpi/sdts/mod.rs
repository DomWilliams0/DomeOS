mod fadt;

pub use fadt::{Fadt, FadtRevision1, FadtRevision3};

pub trait DescriptionTable {
    const SIGNATURE: &'static str;
}
