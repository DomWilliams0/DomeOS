mod fadt;

pub use fadt::{Fadt, FadtRevision1, FadtRevision2};

pub trait DescriptionTable {
    const SIGNATURE: &'static str;
}
