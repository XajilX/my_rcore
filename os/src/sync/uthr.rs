// UniTHReadCell

use core::{cell::{RefCell, RefMut, UnsafeCell}, ops::{Deref, DerefMut}};

use lazy_static::lazy_static;
use riscv::register::sstatus;

pub struct UThrCell<T> {
    inner: RefCell<T>
}

unsafe impl<T> Sync for UThrCell<T> {}

pub struct UThrRefMut<'a, T>(Option<RefMut<'a, T>>);

impl<T> UThrCell<T> {
    pub unsafe fn new(value: T) -> Self {
        Self { inner: RefCell::new(value) }
    }
    pub fn get_refmut(&self) -> UThrRefMut<'_, T> {
        INTR_STATE.get_mut().enter();
        UThrRefMut(Some(self.inner.borrow_mut()))
    }
    pub fn then<F, V>(&self, f: F) -> V where
        F: FnOnce(&mut T) -> V
    {
        let mut inner = self.get_refmut();
        f(&mut inner)
    }
}

impl<'a, T> Drop for UThrRefMut<'a, T>  {
    fn drop(&mut self) {
        self.0 = None;
        INTR_STATE.get_mut().exit();
    }
}

impl<'a, T> Deref for UThrRefMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap().deref()
    }
}

impl<'a, T> DerefMut for UThrRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap().deref_mut()
    }
}

struct UThrCellRaw<T> {
    inner: UnsafeCell<T>
}

unsafe impl<T> Sync for UThrCellRaw<T>  {}

impl<T> UThrCellRaw<T> {
    pub unsafe fn new(value: T) -> Self {
        Self { inner: UnsafeCell::new(value) }
    }
    pub fn get_mut(&self) -> &mut T {
        unsafe { &mut *(self.inner.get()) }
    }
}

struct IntrState {
    nested_level: usize,
    sie_state: bool
}

impl IntrState {
    pub fn new() -> Self {
        Self {
            nested_level: 0,
            sie_state: false
        }
    }

    pub fn enter(&mut self) {
        let sie = sstatus::read().sie();
        unsafe {
            sstatus::clear_sie();
        }
        if self.nested_level == 0 {
            self.sie_state = sie;
        }
        self.nested_level += 1;
    }

    pub fn exit(&mut self) {
        self.nested_level -= 1;
        if self.nested_level == 0 && self.sie_state {
            unsafe { sstatus::set_sie(); }
        }
    }
}

lazy_static! {
    static ref INTR_STATE: UThrCellRaw<IntrState> = unsafe {
        UThrCellRaw::new(IntrState::new())
    };
}