use core::cell::{Cell, UnsafeCell};
use core::fmt::Debug;
use core::hint::unreachable_unchecked;
use core::ops::{Deref, DerefMut};

#[derive(Copy, Clone)]
enum BorrowState {
    Unused,
    Locked,
    Reading { locked: bool },
    Writing,
}

/// This is a reimplementation of [`RefCell`](core::cell::RefCell) with an additional flag
/// that tracks if mutable access was granted while a `canopen` service tried
/// to lock the contained value.
///
/// Compared to `RefCell` there are two notable differences:
/// * only one [`Ref`] can be active at a time (consecutive calls to [`borrow`](ODCell::borrow) will panic)
/// * a `locked` flag indicates if a `canopen` service expects the contained value not to change because it is in the middle of a multi-step read/write process
///
/// The state of this `locked` flag can be queried with the [`is_locked`](ODCell::is_locked) method.
///
/// Acquiring mutable access with [`try_borrow_mut`](ODCell::try_borrow_mut) or [`borrow_mut`](ODCell::borrow_mut)
/// will forcibly unlock the Cell.
/// If a canopen service was relying on the variable not being changed,
/// it will see the locked flag not being set and abort the operation it was doing.
pub struct ODCell<T: ?Sized> {
    borrow: Cell<BorrowState>,
    value: UnsafeCell<T>,
}

#[derive(Debug)]
pub struct BorrowError {}

impl<T> ODCell<T> {
    #[inline]
    pub const fn new(value: T) -> ODCell<T> {
        ODCell {
            value: UnsafeCell::new(value),
            borrow: Cell::new(BorrowState::Unused),
        }
    }

    #[inline]
    pub fn into_inner(self) -> T {
        // Since this function takes `self` (the `RefCell`) by value, the
        // compiler statically verifies that it is not currently borrowed.
        self.value.into_inner()
    }
}

impl<T: ?Sized> ODCell<T> {
    #[inline]
    #[track_caller]
    pub fn borrow(&self) -> Ref<'_, T> {
        self.try_borrow().expect("already mutably borrowed")
    }

    #[inline]
    pub fn try_borrow(&self) -> Result<Ref<'_, T>, BorrowError> {
        match BorrowRef::new(&self.borrow) {
            Some(b) => {
                // SAFETY: `BorrowRef` ensures that there is only immutable access
                // to the value while borrowed.
                Ok(Ref {
                    value: unsafe { &*self.value.get() },
                    _borrow: b,
                })
            }
            None => Err(BorrowError {}),
        }
    }

    #[inline]
    #[track_caller]
    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.try_borrow_mut().expect("already borrowed")
    }

    #[inline]
    pub fn try_borrow_mut(&self) -> Result<RefMut<'_, T>, BorrowError> {
        match BorrowRefMut::new(&self.borrow) {
            Some(b) => {
                // SAFETY: `BorrowRefMut` guarantees unique access.
                Ok(RefMut {
                    value: unsafe { &mut *self.value.get() },
                    _borrow: b,
                })
            }
            None => Err(BorrowError {}),
        }
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.value.get_mut()
    }

    #[inline]
    pub fn is_locked(&self) -> bool {
        matches!(
            self.borrow.get(),
            BorrowState::Locked | BorrowState::Reading { locked: true }
        )
    }
}

unsafe impl<T: ?Sized> Send for ODCell<T> where T: Send {}

struct BorrowRef<'b> {
    borrow: &'b Cell<BorrowState>,
}

impl<'b> BorrowRef<'b> {
    #[inline]
    fn new(borrow: &'b Cell<BorrowState>) -> Option<BorrowRef<'b>> {
        match borrow.get() {
            BorrowState::Unused => {
                borrow.set(BorrowState::Reading { locked: false });
                Some(BorrowRef { borrow })
            }
            BorrowState::Locked => {
                borrow.set(BorrowState::Reading { locked: true });
                Some(BorrowRef { borrow })
            }
            _ => None,
        }
    }
}

impl Drop for BorrowRef<'_> {
    #[inline]
    fn drop(&mut self) {
        match self.borrow.get() {
            BorrowState::Reading { locked } => {
                if locked {
                    self.borrow.set(BorrowState::Locked);
                } else {
                    self.borrow.set(BorrowState::Unused);
                }
            }
            _ => {
                debug_assert!(false);
                unsafe { unreachable_unchecked() }
            }
        }
    }
}

pub struct Ref<'b, T: ?Sized> {
    value: &'b T,
    _borrow: BorrowRef<'b>,
}

impl<T: ?Sized> Ref<'_, T> {
    pub(crate) fn lock(self) {
        // this produces the same result as dropping self with the locked flag always set
        debug_assert!(matches!(
            self._borrow.borrow.get(),
            BorrowState::Reading { .. }
        ));
        self._borrow.borrow.set(BorrowState::Locked);
        core::mem::forget(self);
    }
    pub(crate) fn unlock(self) {
        // this produces the same result as dropping self with the locked flag always unset
        debug_assert!(matches!(
            self._borrow.borrow.get(),
            BorrowState::Reading { .. }
        ));
        self._borrow.borrow.set(BorrowState::Unused);
        core::mem::forget(self);
    }
}

impl<T: ?Sized> Deref for Ref<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.value
    }
}

struct BorrowRefMut<'b> {
    borrow: &'b Cell<BorrowState>,
}

impl Drop for BorrowRefMut<'_> {
    #[inline]
    fn drop(&mut self) {
        let borrow = self.borrow.get();
        debug_assert!(matches!(borrow, BorrowState::Writing));
        self.borrow.set(BorrowState::Unused);
    }
}

impl<'b> BorrowRefMut<'b> {
    #[inline]
    fn new(borrow: &'b Cell<BorrowState>) -> Option<BorrowRefMut<'b>> {
        match borrow.get() {
            BorrowState::Unused | BorrowState::Locked => {
                borrow.set(BorrowState::Writing);
                Some(BorrowRefMut { borrow })
            }
            _ => None,
        }
    }
}

pub struct RefMut<'b, T: ?Sized> {
    value: &'b mut T,
    _borrow: BorrowRefMut<'b>,
}

impl<T: ?Sized> RefMut<'_, T> {
    pub(crate) fn lock(self) {
        // this produces the same result as dropping self
        // and then switching from BorrowState::Unused to BorrowState::Locked
        debug_assert!(matches!(self._borrow.borrow.get(), BorrowState::Writing));
        self._borrow.borrow.set(BorrowState::Locked);
        core::mem::forget(self);
    }
}

impl<T: ?Sized> Deref for RefMut<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.value
    }
}

impl<T: ?Sized> DerefMut for RefMut<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        self.value
    }
}
