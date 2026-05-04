#!/bin/bash
# ESP32-C3 烧录脚本

# Override if the port changes after replug: ESPFLASH_PORT=/dev/cu.usbmodemXXXX ./flash.sh
PORT="${ESPFLASH_PORT:-/dev/cu.usbmodem21101}"
CHIP="esp32c3"
FIRMWARE="target/riscv32imc-unknown-none-elf/debug/firmware"

echo "正在烧录到 $PORT..."
echo "如果失败，请按住 BOOT 按钮，按一下 RESET，然后松开 BOOT"
echo ""

# 方法1：默认设置
echo "尝试方法1：默认设置"
espflash flash --chip $CHIP --monitor $FIRMWARE --port $PORT

# 如果失败，尝试方法2
if [ $? -ne 0 ]; then
    echo ""
    echo "尝试方法2：使用 USB 重置"
    espflash flash --chip $CHIP --monitor $FIRMWARE --port $PORT --before usb-reset
fi

# 如果还失败，尝试方法3
if [ $? -ne 0 ]; then
    echo ""
    echo "尝试方法3：低波特率"
    espflash flash --chip $CHIP --monitor $FIRMWARE --port $PORT --baud 115200
fi

# 如果还失败，尝试方法4
if [ $? -ne 0 ]; then
    echo ""
    echo "尝试方法4：不使用 stub"
    espflash flash --chip $CHIP --monitor $FIRMWARE --port $PORT --no-stub
fi
