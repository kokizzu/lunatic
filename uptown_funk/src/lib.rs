pub mod memory;
pub mod state;
pub mod types;
#[cfg(feature = "vm-wasmer")]
pub mod wasmer;

use std::cell::{Ref, RefCell, RefMut};
use std::convert::Into;
use std::fmt::Debug;

pub use smallvec::SmallVec;
pub use uptown_funk_macro::host_functions;

/// Provides access to the instance execution environment.
pub trait Executor {
    /// Execute `Future` f.
    #[cfg(feature = "async")]
    fn async_<R, F>(&self, f: F) -> R
    where
        F: std::future::Future<Output = R>;

    /// Get mutable access to the instance memory.
    fn memory(&self) -> memory::Memory;
}

pub trait HostFunctions {
    #[cfg(feature = "vm-wasmtime")]
    fn add_to_linker<E>(self, instance: E, linker: &mut wasmtime::Linker)
    where
        E: Executor + 'static;

    #[cfg(feature = "vm-wasmer")]
    fn add_to_wasmer_linker<E>(
        self,
        instance: E,
        linker: &mut wasmer::WasmerLinker,
        store: &::wasmer::Store,
    ) where
        E: Executor + 'static;
}

pub trait FromWasm {
    type From;
    type State;

    fn from(
        state: &mut Self::State,
        instance: &impl Executor,
        from: Self::From,
    ) -> Result<Self, Trap>
    where
        Self: Sized;
}

pub trait ToWasm {
    type To;
    type State;

    fn to(
        state: &mut Self::State,
        instance: &impl Executor,
        host_value: Self,
    ) -> Result<Self::To, Trap>;
}

pub struct StateWrapper<S, E: Executor> {
    state: RefCell<S>,
    env: E,
}

impl<S, E: Executor> StateWrapper<S, E> {
    pub fn new(state: S, instance: E) -> Self {
        Self {
            state: RefCell::new(state),
            env: instance,
        }
    }

    pub fn borrow_state(&self) -> Ref<S> {
        self.state.borrow()
    }

    pub fn borrow_state_mut(&self) -> RefMut<S> {
        self.state.borrow_mut()
    }

    pub fn instance(&self) -> &E {
        &self.env
    }

    pub fn memory(&self) -> memory::Memory {
        self.env.memory()
    }
}

#[cfg_attr(feature = "vm-wasmer", error("{message}"))]
#[cfg_attr(feature = "vm-wasmer", derive(thiserror::Error))]
#[derive(Debug)]
pub struct Trap {
    message: String,
}

impl Trap {
    pub fn new<I: Into<String>>(message: I) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn try_option<R: Debug>(result: Option<R>) -> Result<R, Trap> {
        match result {
            Some(r) => Ok(r),
            None => Err(Trap::new(
                "Host function trapped: Memory location not inside wasm guest",
            )),
        }
    }

    pub fn try_result<R: Debug, E: Debug>(result: Result<R, E>) -> Result<R, Trap> {
        match result {
            Ok(r) => Ok(r),
            Err(_) => {
                let message = format!("Host function trapped: {:?}", result);
                Err(Trap::new(message))
            }
        }
    }
}

#[cfg(feature = "vm-wasmtime")]
impl From<Trap> for wasmtime::Trap {
    fn from(trap: Trap) -> Self {
        wasmtime::Trap::new(trap.message)
    }
}

#[repr(C)]
pub struct IoVecT {
    pub ptr: u32,
    pub len: u32,
}
