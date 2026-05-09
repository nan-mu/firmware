// use embedded_storage::Storage;
use core::ptr;
use esp_hal::peripherals::FLASH;
use esp_storage::FlashStorage;
use log::{debug, info};

#[unsafe(link_section = ".iram1.hotload_area")]
#[unsafe(no_mangle)]
static mut XDP: [u8; 4096] = [0u8; 4096];

pub fn load(flash: FLASH<'static>) {
    let mut flash = FlashStorage::new(flash);
    let mut buffer = [0u8; esp_bootloader_esp_idf::partitions::PARTITION_TABLE_MAX_LEN];
    let pt =
        esp_bootloader_esp_idf::partitions::read_partition_table(&mut flash, &mut buffer).unwrap();
    // List all partitions - this is just FYI
    for part in pt.iter() {
        info!("{:?}", part);
    }

    let xdp_with_footer = include_bytes!("/Users/nan/bs/firmware/payload.bin");

    unsafe {
        // 1. 解析末尾 4 字节的 Footer
        // 指针指向 buffer 的最后 4 个字节

        let bin_size = xdp_with_footer.len();
        let bss_size = 0;

        info!(
            "[AOT] 检测到 Bin: {} 字节, BSS: {} 字节",
            bin_size, bss_size
        );

        // 2. 拷贝代码和数据段 (.text, .rodata, .data)
        // 直接从 network_payload.dat 的开头拷贝 bin_size 个字节
        let xdp_ptr = ptr::addr_of_mut!(XDP) as *mut u8;
        ptr::copy_nonoverlapping(xdp_with_footer.as_ptr(), xdp_ptr, bin_size);

        // 3. 初始化并清零 BSS 区域
        // 在 bin 结束后的位置开始，清空 bss_size 长度的内存
        ptr::write_bytes(xdp_ptr.add(bin_size), 0, bss_size);

        info!("[AOT] 加载完成");
    }
}

#[repr(C)]
pub struct XdpContext {
    data: *const u8,
    data_end: *const u8,
}

pub fn xdp(data: &[u8]) -> i32 {
    let xdp_ptr = ptr::addr_of_mut!(XDP) as *mut u8;
    unsafe {
        core::arch::asm!("fence.i");
        
        let entry: extern "C" fn(*const XdpContext) -> i32 = 
            core::mem::transmute(xdp_ptr);
        
        let ctx = XdpContext {
            data: data.as_ptr(),
            data_end: data.as_ptr().add(data.len()),
        };
        
        let res = entry(&ctx as *const _);
        debug!("[AOT] 程序返回值: {}", res);
        res
    }
}
