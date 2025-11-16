use axerrno::AxError;
use axtask::current;
use starry_core::task::AsThread;
use starry_vm::VmPtr;

/// Minimal implementation of the rseq syscall registration.
///
/// This implementation only supports registration/unregistration via the
/// first argument (addr) and the flags argument. It stores the user pointer
/// in the current thread structure so kernel-side users can inspect it.
///
/// C prototype (simplified):
/// long rseq(void *addr, uint32_t len, int flags, uint32_t sig);
pub fn sys_rseq(addr: *mut u8, len: usize, flags: u32, sig: u32) -> Result<isize, AxError> {
    debug!(
        "sys_rseq <= addr: {:?}, len: {}, flags: {}, sig: {}",
        addr, len, flags, sig
    );

    // According to Linux, addr == NULL and len == 0 unregisters.
    // Validate inputs: len should be either 0 (unregister) or match expected header
    // size. For simplicity accept any non-zero len up to a reasonable limit.
    if addr.is_null() {
        if len != 0 {
            return Err(AxError::InvalidInput);
        }
        // unregister
        current().as_thread().set_rseq_area(0);
        return Ok(0);
    }

    if len == 0 {
        return Err(AxError::InvalidInput);
    }

    // // Check that the user pointer is readable/writable (we only need the
    // address). // Try to read one byte to ensure the area is valid.
    if addr.vm_read().is_err() {
        return Err(AxError::InvalidInput);
    }

    // // Store the user address in the thread.
    current().as_thread().set_rseq_area(addr.addr());

    Ok(0)
}
