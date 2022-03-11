#!/bin/sh -e

for swapspace in $(swapon -s | awk '/zram/{print $1}'); do
    swapoff $swapspace
done

modprobe -r zram

ALLOCATION=$((2 * 1024 * 1024))

modprobe zram

echo zstd > /sys/block/zram0/comp_algorithm

echo $ALLOCATION > /sys/block/zram0/disksize

dd if=/dev/zero of=/dev/zram0 bs=$((1024 * 1024)) count=2

mkswap /dev/zram0

swapon -p 100 /dev/zram0
