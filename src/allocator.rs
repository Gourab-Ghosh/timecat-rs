use super::*;

#[cfg(feature = "extern_alloc")]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(feature = "extern_alloc")]
pub static GLOBAL_ALLOCATOR: &str = "mimalloc::MiMalloc";
#[cfg(not(feature = "extern_alloc"))]
pub static GLOBAL_ALLOCATOR: &str = "Default Rust Allocator";
