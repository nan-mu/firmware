SECTIONS
{
    /* 将 helper_dispatcher 放在 IRAM 中间偏后的位置 */
    .iram1.helper_dispatcher 0x403A0000 : 
    {
        . = ALIGN(4);
        KEEP(*(.iram1.helper_dispatcher))
    } > IRAM

    /* hotload_area 紧随其后，从 0x403A1000 开始 (给 helper 预留 4KB) */
    .iram1.hotload_area 0x403A1000 :
    {
        . = ALIGN(4);
        KEEP(*(.iram1.hotload_area))
        . = . + 4096;  /* 预留 4096 字节 */
    } > IRAM
}