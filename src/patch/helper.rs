use log::{debug, info};

#[unsafe(no_mangle)]
#[unsafe(link_section = ".iram1.helper_dispatcher")]
pub extern "C" fn helper_jump(idx: u32, arg1: usize, arg2: usize) -> usize {
    match idx {
        1 => log(arg1, arg2),
        _ => 1,
    }
}

fn log(arg1: usize, arg2: usize) -> usize {
    let ptr = arg1 as *const u8;
    let len = arg2;

    let slice = unsafe { core::slice::from_raw_parts(ptr, len as usize) };
    debug!(
        "[Helpet] Log Len Check: {}, Content: {:?}",
        slice.len(),
        slice
    );
    let slice = core::str::from_utf8(slice).unwrap_or("Decode Error");
    info!("{}", slice);
    0
}
