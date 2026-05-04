#!/bin/bash
# 诊断脚本

echo "=== ESP32-C3 诊断信息 ==="
echo ""

echo "1. 检查串口设备："
ls -l /dev/cu.usbmodem* 2>&1
echo ""

echo "2. espflash 版本："
espflash --version
echo ""

echo "3. 尝试连接设备（5秒超时）："
timeout 5 espflash board-info --port /dev/cu.usbmodem57280376921 2>&1 || echo "连接超时或失败"
echo ""

echo "4. 固件文件信息："
ls -lh target/riscv32imc-unknown-none-elf/debug/firmware 2>&1
echo ""

echo "=== 诊断完成 ==="
