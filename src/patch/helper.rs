use ::log::info;

mod log;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".iram1.helper_dispatcher")]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn helper_dispatcher(idx: usize, fmt_ptr: usize, fmt_len: usize, arg1: usize, arg2: usize) -> usize {
    info!("[helper_dispatcher] Called with idx={}, fmt_ptr={:#x}, fmt_len={}, arg1={:#x}, arg2={:#x}", 
          idx, fmt_ptr, fmt_len, arg1, arg2);
    
    match idx {
        1 => {
            let fmt = unsafe { core::slice::from_raw_parts(fmt_ptr as *const u8, fmt_len) };
            info!("[helper_dispatcher] Format string: {:?}", core::str::from_utf8(fmt));
            log(fmt, &[arg1])
        }
        2 => {
            let fmt = unsafe { core::slice::from_raw_parts(fmt_ptr as *const u8, fmt_len) };
            log(fmt, &[arg1, arg2])
        }
        _ => {
            info!("[helper_dispatcher] Invalid idx: {}", idx);
            usize::MAX
        }
    }
}

fn log(fmt: &[u8], args: &[usize]) -> usize {
    let tokens = log::parse_bpf_format(fmt);
    info!("[helper_dispatcher] Parsed {} tokens", tokens.len());
    let msg = log::zip_format(tokens, args);
    info!("XDP: {}", msg);
    0
}