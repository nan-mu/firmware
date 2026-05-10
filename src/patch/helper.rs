use ::log::info;

mod log;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".iram1.helper_dispatcher")]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn helper_jump(idx: usize, arg: [usize; 5]) -> usize {
    match idx {
        0 | 1 | 2 | 3 => {
            let fmt = unsafe { core::slice::from_raw_parts(arg[0] as *const u8, arg[1]) };
            let arg = &arg[2..(2 + idx)];
            log(fmt, arg)
        }
        _ => usize::MAX,
    }
}

fn log(fmt: &[u8], arg: &[usize]) -> usize {
    let msg = log::zip_format(log::parse_bpf_format(fmt), arg);
    info!("XDP: {}", msg);
    0
}