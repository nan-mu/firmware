use ::log::{error, info};

mod log;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".iram1.helper_dispatcher")]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn helper_dispatcher(
    idx: usize,
    fmt_ptr: usize,
    fmt_len: usize,
    arg1: usize,
    arg2: usize,
) -> usize {
    // 读取格式字符串
    let fmt = unsafe { core::slice::from_raw_parts(fmt_ptr as *const u8, fmt_len) };

    match idx {
        0 => log(fmt, &[]),
        1 => log(fmt, &[arg1]),
        2 => log(fmt, &[arg1, arg2]),
        _ => {
            error!("[helper_dispatcher] Invalid idx: {}", idx);
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
