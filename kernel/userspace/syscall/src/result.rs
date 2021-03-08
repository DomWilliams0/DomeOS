use ux::u63;

use crate::error::SyscallError;

// TODO feature gate "kernel"

/// Must be representable only as a positive u64
pub trait SyscallReturnable: From<SyscallOkResult> + Into<SyscallOkResult> {}

/// Result returned by syscall in rax
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct SyscallResult(u64);

/// Positive u64 result
#[repr(transparent)]
pub struct SyscallOkResult(pub u63);

#[cfg(any(feature = "userspace", test))]
pub fn parse_syscall_result<T: From<SyscallOkResult>>(
    returned: SyscallResult,
) -> Result<T, SyscallError> {
    use core::convert::TryFrom;

    let signed: i64 = unsafe { core::mem::transmute(returned) };
    if signed.is_negative() {
        let err = SyscallError::try_from(signed.abs() as u64).unwrap_or(SyscallError::UnknownError);
        Err(err)
    } else {
        // TODO ideally unsafe version to avoid the assert
        let positive = u63::new(returned.0);
        Ok(T::from(SyscallOkResult(positive)))
    }
}

/// >= 0
#[inline]
fn is_ok(val: u64) -> bool {
    let signed: i64 = unsafe { core::mem::transmute(val) };
    !signed.is_negative()
}

impl SyscallResult {
    /// Err if value cannot be represented as a successful syscall result
    pub fn try_ok<T: Into<u64> + Clone>(val: T) -> Result<Self, T> {
        let int = val.clone().into();
        if is_ok(int) {
            let ok = SyscallOkResult(u63::new(int));
            Ok(ok.into())
        } else {
            Err(val)
        }
    }

    pub const fn error(error: SyscallError) -> Self {
        let negative = -(error as i64);
        Self(unsafe { core::mem::transmute::<i64, u64>(negative) })
    }

    pub const fn to_u64(&self) -> u64 {
        self.0
    }
}

impl<T: Into<SyscallOkResult>> From<Result<T, SyscallError>> for SyscallResult {
    fn from(result: Result<T, SyscallError>) -> Self {
        match result {
            Ok(ok) => Self(ok.into().into()),
            Err(err) => Self::error(err),
        }
    }
}

impl From<SyscallError> for SyscallResult {
    #[inline]
    fn from(err: SyscallError) -> Self {
        Self::error(err)
    }
}

impl From<SyscallOkResult> for SyscallResult {
    #[inline]
    fn from(ok: SyscallOkResult) -> Self {
        Self(u64::from(ok))
    }
}

impl<T: From<SyscallOkResult> + Into<SyscallOkResult>> SyscallReturnable for T {}

impl From<SyscallOkResult> for u64 {
    #[inline]
    fn from(val: SyscallOkResult) -> Self {
        let positive = u64::from(val.0);
        debug_assert_eq!(positive.to_le_bytes()[7], 0);
        positive
    }
}

// --------- trivial conversions --------- //

impl From<u32> for SyscallOkResult {
    #[inline]
    fn from(val: u32) -> Self {
        Self(u63::from(val))
    }
}

impl From<u16> for SyscallOkResult {
    #[inline]
    fn from(val: u16) -> Self {
        Self(u63::from(val))
    }
}

impl From<u8> for SyscallOkResult {
    #[inline]
    fn from(val: u8) -> Self {
        Self(u63::from(val))
    }
}

impl From<()> for SyscallOkResult {
    #[inline]
    fn from(_: ()) -> Self {
        Self::from(0u8)
    }
}

#[cfg(test)]
mod tests {
    use core::fmt::Debug;

    use super::*;

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    struct Nice(u32, u32);

    impl From<Nice> for SyscallOkResult {
        fn from(nice: Nice) -> SyscallOkResult {
            let full = ((nice.0 as u64) << 32) | nice.1 as u64;
            let inner = u63::new(full); // asserts bit 63 is 0
            SyscallOkResult(inner)
        }
    }

    impl From<SyscallOkResult> for Nice {
        fn from(val: SyscallOkResult) -> Self {
            let val = u64::from(val.0);
            let hi = (val >> 32) as u32;
            let lo = (val & 0xffffffff) as u32;
            Self(hi, lo)
        }
    }

    impl From<SyscallOkResult> for u32 {
        fn from(result: SyscallOkResult) -> Self {
            u64::from(result.0) as u32
        }
    }

    fn check_ok<T>(val: T)
    where
        T: SyscallReturnable + Copy + Eq + Debug,
    {
        let ok: SyscallOkResult = val.into();
        let result = SyscallResult::from(ok);
        assert_eq!(core::mem::size_of_val(&result), 8);

        let parsed = parse_syscall_result::<T>(result);
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap(), val)
    }

    #[test]
    fn parsing() {
        let err = SyscallResult::from(SyscallError::NotImplemented);

        check_ok(5u32);
        check_ok(u32::MAX);
        check_ok(Nice(10, 20));

        assert!(SyscallResult::try_ok(u64::MAX).is_err());

        let res = SyscallResult::try_ok(500_u64).unwrap();
        assert_eq!(parse_syscall_result::<u64>(res).unwrap(), 500);

        assert!(matches!(
            parse_syscall_result::<Nice>(err),
            Err(SyscallError::NotImplemented)
        ));
    }

    #[test]
    #[should_panic]
    fn unrepresentable_ok() {
        // bit 63 is 1, meaning the underlying int is negative and treated as an error
        check_ok(Nice(u32::MAX, 20));
    }

    #[test]
    fn from_result() {
        let ok = Ok(());
        let err = Result::<(), SyscallError>::Err(SyscallError::InvalidArguments);

        impl From<SyscallOkResult> for () {
            fn from(_: SyscallOkResult) -> Self {}
        }

        assert!(matches!(
            parse_syscall_result::<()>(SyscallResult::from(ok)),
            Ok(())
        ));
        assert!(matches!(
            parse_syscall_result::<()>(SyscallResult::from(err)),
            Err(SyscallError::InvalidArguments)
        ));
    }
}
