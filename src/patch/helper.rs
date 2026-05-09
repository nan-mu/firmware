use log::{debug, error, info};

#[unsafe(no_mangle)]
#[unsafe(link_section = ".iram1.helper_dispatcher")]
pub extern "C" fn helper_jump(idx: u32, arg1: usize, arg2: usize, arg3: usize) -> usize {
    match idx {
        1 => log(arg1, arg2, arg3),
        _ => 1,
    }
}

fn log(kind: usize, ptr: usize, len: usize) -> usize {
    match kind {
        0 => {
            let ptr = ptr as *const u8;
            let slice = unsafe { core::slice::from_raw_parts(ptr, len as usize) };
            debug!("[Helpet] Log str: {}, Content: {:?}", slice.len(), slice);
            let slice = core::str::from_utf8(slice).unwrap_or("Decode Error");
            info!("{}", slice);
            0
        }
        1 => {
            let imm = ptr;
            debug!("[Helpet] Log uimm: {}", imm);
            0
        }
        2 => {
            let imm = ptr as isize;
            debug!("[Helpet] Log uimm: {}", imm);
            0
        }
        _ => {
            error!("Undefined log kind");
            usize::MAX
        }
    }
}
