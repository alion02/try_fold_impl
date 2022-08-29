#![feature(try_trait_v2, try_blocks)]

use std::ops::{ControlFlow, FromResidual, Try};

#[doc(hidden)]
pub struct Break<T>(pub T);

impl<T> FromResidual<T> for Break<T> {
    fn from_residual(residual: T) -> Self {
        Self(residual)
    }
}

impl<T> Try for Break<T> {
    type Output = T;
    type Residual = T;

    fn from_output(output: T) -> Self {
        Self(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        ControlFlow::Break(self.0)
    }
}

#[doc(hidden)]
pub struct Continue<T>(pub T);

impl<T> FromResidual<T> for Continue<T> {
    fn from_residual(residual: T) -> Self {
        Self(residual)
    }
}

impl<T> Try for Continue<T> {
    type Output = T;
    type Residual = T;

    fn from_output(output: T) -> Self {
        Self(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        ControlFlow::Continue(self.0)
    }
}

#[macro_export]
macro_rules! try_fold {
    ($(#[$($m:meta),*])? ($self:ident, $acc:ident, $f:ident) $body:block) => {
        $($(#[$m])*)?
        fn try_fold<B, F, R>(&mut $self, mut $acc: B, mut $f: F) -> R
        where
            F: FnMut(B, Self::Item) -> R,
            R: std::ops::Try<Output = B>,
        {
            $body
        }

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            self.try_fold(None, |_, next| $crate::Break(Some(next))).0
        }

        #[inline]
        fn fold<B, F>(mut self, init: B, mut f: F) -> B
        where
            F: FnMut(B, Self::Item) -> B,
        {
            self.try_fold(init, |acc, next| $crate::Continue(f(acc, next))).0
        }
    }
}

#[cfg(test)]
mod tests {
    struct Test1(u8);

    impl Iterator for Test1 {
        type Item = u8;

        try_fold!((self, acc, f) {
            while self.0 > 0 {
                let new = f(acc, self.0.wrapping_mul(173));
                self.0 -= 1;
                acc = new?;
            }
            try { acc }
        });

        fn size_hint(&self) -> (usize, Option<usize>) {
            let s = self.0 as usize;
            (s, Some(s))
        }
    }
    struct Test2;

    impl Iterator for Test2 {
        type Item = ();

        try_fold!(#[inline, cold] (self, _acc, _f) {
            unimplemented!()
        });
    }

    #[test]
    fn impls() {
        let mut t = Test1(4);

        assert_eq!(t.next(), Some(173u8.wrapping_mul(4)));
        assert_eq!(t.next(), Some(173u8.wrapping_mul(3)));

        assert_eq!(t.size_hint(), (2, Some(2)));

        assert_eq!(
            t.map(u32::from).product::<u32>(),
            173u8.wrapping_mul(2) as u32 * 173
        );
    }
}
