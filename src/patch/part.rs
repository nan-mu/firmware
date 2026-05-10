// use embedded_storage::Storage;
use core::ptr;
use esp_hal::peripherals::FLASH;
// use esp_storage::FlashStorage;
use log::info;

#[unsafe(link_section = ".iram1.hotload_area")]
#[unsafe(no_mangle)]
static mut XDP: [u8; 4096] = [0u8; 4096];

pub fn load(_flash: FLASH<'static>) {
    // let mut flash = FlashStorage::new(flash);
    // let mut buffer = [0u8; esp_bootloader_esp_idf::partitions::PARTITION_TABLE_MAX_LEN];
    // let pt =
    //     esp_bootloader_esp_idf::partitions::read_partition_table(&mut flash, &mut buffer).unwrap();
    // // List all partitions - this is just FYI
    // for part in pt.iter() {
    //     info!("{:?}", part);
    // }

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

        // 在栈上创建 ctx
        let mut ctx = XdpContext {
            data: data.as_ptr(),
            data_end: data.as_ptr().add(data.len()),
        };

        let ctx_ptr = &mut ctx as *mut XdpContext;

        info!(
            "[AOT] 调用 XDP，ctx_ptr={:p}, data={:p}, data_end={:p}",
            ctx_ptr, ctx.data, ctx.data_end
        );

        // 使用内联汇编调用，确保寄存器正确
        let res: i32;
        core::arch::asm!(
            "jalr {entry}",
            entry = in(reg) xdp_ptr,
            in("a0") ctx_ptr,
            lateout("a0") res,
            // 标记所有可能被破坏的寄存器
            clobber_abi("C"),
        );

        info!("[AOT] XDP返回: {}", res);
        res
    }
}
