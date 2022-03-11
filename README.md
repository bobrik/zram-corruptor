# zram corruption

Build the kernel with the following patch:

* [orphan-zram-page.patch](./orphan-zram-page.patch)

Then run the reproduction commands:

```
sudo ./recycle-zram.sh && sleep 1 && cargo build --release && sudo systemd-run --scope -p MemoryLimit=34M ./target/release/zram-corruptor
```

With any luck you'll be able to see (it might take a few attempts):

```
[  512.651752][ T7285] ------------[ cut here ]------------
[  512.652279][ T7285] WARNING: CPU: 0 PID: 7285 at drivers/block/zram/zram_drv.c:1285 __zram_bvec_read+0x28c/0x2e8 [zram]
[  512.653923][ T7285] Modules linked in: zram zsmalloc kheaders nfsv3 nfs lockd grace sunrpc xt_conntrack nft_chain_nat xt_MASQUERADE nf_nat nf_conntrack_netlink nf_conntrack nf_defrag_ipv6 nf_defrag_ipv4 nft_counter xt_addrtype nft_compat nf_tables nfnetlink bridge stp llc overlay xfs libcrc32c zstd zstd_compress af_packet aes_ce_blk aes_ce_cipher ghash_ce gf128mul virtio_net sha3_ce net_failover sha3_generic failover sha512_ce sha512_arm64 sha2_ce sha256_arm64 virtio_mmio virtio_ring qemu_fw_cfg rtc_pl031 virtio fuse ip_tables x_tables ext4 mbcache crc16 jbd2 nvme nvme_core pci_host_generic pci_host_common unix [last unloaded: zsmalloc]
[  512.659238][ T7285] CPU: 0 PID: 7285 Comm: zram-corruptor Tainted: G        W         5.16.0-ivan #1 0877d306c6dc0716835d43cafe4399473d09e406
[  512.660413][ T7285] Hardware name: linux,dummy-virt (DT)
[  512.661077][ T7285] pstate: 80400005 (Nzcv daif +PAN -UAO -TCO -DIT -SSBS BTYPE=--)
[  512.661788][ T7285] pc : __zram_bvec_read+0x28c/0x2e8 [zram]
[  512.662099][ T7285] lr : zram_bvec_rw+0x70/0x204 [zram]
[  512.662422][ T7285] sp : ffffffc01018bac0
[  512.662720][ T7285] x29: ffffffc01018bae0 x28: ffffff9e4e725280 x27: ffffff9e4e725280
[  512.663122][ T7285] x26: ffffff9e4e725280 x25: 00000000000001f6 x24: 0000000100033e6c
[  512.663601][ T7285] x23: 00000000000001f6 x22: 0000000000000000 x21: fffffffe7a36d840
[  512.664252][ T7285] x20: 00000000000001f6 x19: ffffff9e69423c00 x18: ffffffc010711068
[  512.664812][ T7285] x17: 0000000000000008 x16: ffffffd34aed51bc x15: 0000000000000000
[  512.665507][ T7285] x14: 0000000000000a88 x13: 0000000000000000 x12: 0000000000000000
[  512.666183][ T7285] x11: 0000000100033e6c x10: ffffffc01091d000 x9 : 0000000001000000
[  512.666627][ T7285] x8 : 0000000000002f10 x7 : 80b75f8fb90b52c4 x6 : 051609fe50833de3
[  512.667276][ T7285] x5 : 0000000000000000 x4 : 0000000000000000 x3 : 0000000000000000
[  512.667875][ T7285] x2 : 00000000000001f6 x1 : 00000000000001f6 x0 : ffffffd305b746af
[  512.668483][ T7285] Call trace:
[  512.668682][ T7285]  __zram_bvec_read+0x28c/0x2e8 [zram 745969ed35ea0fb382bfd518d6f70e13966e9b52]
[  512.669405][ T7285]  zram_bvec_rw+0x70/0x204 [zram 745969ed35ea0fb382bfd518d6f70e13966e9b52]
[  512.670066][ T7285]  zram_rw_page+0xb4/0x16c [zram 745969ed35ea0fb382bfd518d6f70e13966e9b52]
[  512.670584][ T7285]  bdev_read_page+0x74/0xac
[  512.670843][ T7285]  swap_readpage+0x5c/0x2e4
[  512.671243][ T7285]  do_swap_page+0x2f4/0x988
[  512.671560][ T7285]  handle_pte_fault+0xcc/0x1fc
[  512.671935][ T7285]  handle_mm_fault+0x284/0x4a8
[  512.672412][ T7285]  do_page_fault+0x274/0x428
[  512.672704][ T7285]  do_translation_fault+0x5c/0xf8
[  512.673083][ T7285]  do_mem_abort+0x50/0xc8
[  512.673293][ T7285]  el0_da+0x3c/0x74
[  512.673549][ T7285]  el0t_64_sync_handler+0xc4/0xec
[  512.673972][ T7285]  el0t_64_sync+0x1a4/0x1a8
[  512.674495][ T7285] ---[ end trace cf983b7507c20343 ]---
[  512.675359][ T7285] zram: Page 502 read from zram without previous write
```

You can use `bpftrace` to find what happened:

```
sudo bpftrace -e 'kprobe:zram_free_page { printf("zram_free_page  index = %u [cpu = %d, tid = %d]\n%s\n", arg1, cpu, tid, kstack(16)); } kprobe:__zram_bvec_write { printf("zram_bvec_write index = %u [cpu = %d, tid = %d]\n", arg2, cpu, tid) } kprobe:__zram_bvec_read { printf("zram_bvec_read  index = %u [cpu = %d, tid = %d]\n", arg2, cpu, tid) }' > derp.txt
```

You might need to run things in the following order (possibly a few times):

1. Run `sudo ./recycle-zram.sh`
2. Run `bpftrace` command above
3. Run `cargo build --release && sudo systemd-run` to replicate

* [derp.txt](./derp.txt)

```
$ grep ' = 502' derp.txt
zram_bvec_write index = 502 [cpu = 3, tid = 7286]
zram_free_page  index = 502 [cpu = 3, tid = 7286]
zram_bvec_read  index = 502 [cpu = 3, tid = 7286]
zram_free_page  index = 502 [cpu = 3, tid = 7286] <-- problematic free
zram_bvec_read  index = 502 [cpu = 0, tid = 7285] <-- problematic read
```

In more detail:

```
zram_bvec_write index = 502 [cpu = 3, tid = 7286]
zram_free_page  index = 502 [cpu = 3, tid = 7286]

        zram_free_page+0
        $x.97+32
        zram_rw_page+180
        bdev_write_page+124
        __swap_writepage+116
        swap_writepage+160
        pageout+284
        shrink_page_list+2892
        shrink_inactive_list+688
        shrink_lruvec+360
        shrink_node_memcgs+148
        shrink_node+860
        shrink_zones+368
        do_try_to_free_pages+232
        try_to_free_mem_cgroup_pages+292
        try_charge_memcg+608

zram_bvec_read  index = 502 [cpu = 3, tid = 7286]
zram_free_page  index = 502 [cpu = 3, tid = 7286] <-- problematic free

        zram_free_page+0
        swap_range_free+220
        swap_entry_free+244
        swapcache_free_entries+152
        free_swap_slot+288
        __swap_entry_free+216
        swap_free+108
        do_swap_page+1776
        handle_pte_fault+204
        handle_mm_fault+644
        do_page_fault+628
        do_translation_fault+92
        do_mem_abort+80
        el0_da+60
        el0t_64_sync_handler+196
        el0t_64_sync+420

zram_bvec_read  index = 502 [cpu = 0, tid = 7285] <-- problematic read
```

Here we read a page after its been freed.

Running CPU pinning doesn't replicate the issue:

```
sudo ./recycle-zram.sh && sleep 1 && cargo build --release && sudo systemd-run --scope -p MemoryLimit=34M taskset -c 0 ./target/release/zram-corruptor
```

Neither does running with a single thread in `src/main.rs` (it's two by default).

It can be replicated on 5.17.0-rc7 as well:

```
[  792.341380][T74835] ------------[ cut here ]------------
[  792.343215][T74835] WARNING: CPU: 5 PID: 74835 at drivers/block/zram/zram_drv.c:1298 __zram_bvec_read+0x2a8/0x31c [zram]
[  792.348975][T74835] Modules linked in: zram zsmalloc kheaders veth nfsv3 nfs lockd grace sunrpc xt_conntrack nft_chain_nat xt_MASQUERADE nf_nat nf_conntrack_netlink nf_conntrack nf_defrag_ipv6 nf_defrag_ipv4 xt_addrtype nft_compat nf_tables nfnetlink bridge stp llc overlay xfs libcrc32c zstd zstd_compress aes_ce_blk aes_ce_cipher af_packet ghash_ce gf128mul sha3_ce virtio_net sha3_generic net_failover sha512_ce failover sha512_arm64 sha2_ce sha256_arm64 rtc_pl031 virtio_mmio virtio_ring qemu_fw_cfg virtio fuse ip_tables x_tables ext4 mbcache crc16 jbd2 nvme nvme_core pci_host_generic pci_host_common unix [last unloaded: zsmalloc]
[  792.358113][T74835] CPU: 5 PID: 74835 Comm: hashmapper Tainted: G        W         5.17.0-rc7-ivan #1 9d174e88613b15ea4b912ebcca01465f16e6ea3c
[  792.361861][T74835] Hardware name: linux,dummy-virt (DT)
[  792.362545][T74835] pstate: 80400005 (Nzcv daif +PAN -UAO -TCO -DIT -SSBS BTYPE=--)
[  792.365095][T74835] pc : __zram_bvec_read+0x2a8/0x31c [zram]
[  792.366071][T74835] lr : zram_bvec_rw+0x70/0x20c [zram]
[  792.367718][T74835] sp : ffffffc00ba43ab0
[  792.368973][T74835] x29: ffffffc00ba43ad0 x28: ffffffa202396200 x27: ffffffa20cff4200
[  792.374083][T74835] x26: 00000000000001cb x25: 00000000000001cb x24: 00000001000782e6
[  792.377001][T74835] x23: 00000000000001cb x22: 0000000000000000 x21: fffffffe8b6587c0
[  792.377984][T74835] x20: 00000000000001cb x19: ffffffa20abce000 x18: ffffffc00aa15068
[  792.379058][T74835] x17: 0000000000000008 x16: ffffffd8576df0d4 x15: 0000000000000001
[  792.379914][T74835] x14: 0000000000000000 x13: 00000000000000c0 x12: 0000000000000000
[  792.382436][T74835] x11: 0000000000000000 x10: ffffffc00871f000 x9 : 0000000001000000
[  792.386018][T74835] x8 : 0000000000003960 x7 : e79083464601e40f x6 : 39ba502bc9354410
[  792.391146][T74835] x5 : 0000000000000000 x4 : 0000000000000000 x3 : 0000000000000000
[  792.392527][T74835] x2 : 00000000000001cb x1 : 00000000000001cb x0 : ffffffd7d96406e8
[  792.394098][T74835] Call trace:
[  792.394948][T74835]  __zram_bvec_read+0x2a8/0x31c [zram cafe9fca707c4750fd4edca44dedb86843b7971c]
[  792.396996][T74835]  zram_bvec_rw+0x70/0x20c [zram cafe9fca707c4750fd4edca44dedb86843b7971c]
[  792.397977][T74835]  zram_rw_page+0xb4/0x170 [zram cafe9fca707c4750fd4edca44dedb86843b7971c]
[  792.399947][T74835]  bdev_read_page+0x74/0xac
[  792.400620][T74835]  swap_readpage+0x60/0x328
[  792.400844][T74835]  do_swap_page+0x438/0x904
[  792.401014][T74835]  handle_pte_fault+0xcc/0x1fc
[  792.401216][T74835]  handle_mm_fault+0x284/0x4a8
[  792.402136][T74835]  do_page_fault+0x274/0x428
[  792.403133][T74835]  do_translation_fault+0x5c/0xf8
[  792.404704][T74835]  do_mem_abort+0x50/0x100
[  792.404877][T74835]  el0_da+0x3c/0x74
[  792.405019][T74835]  el0t_64_sync_handler+0xc4/0xec
[  792.409809][T74835]  el0t_64_sync+0x1a4/0x1a8
[  792.410132][T74835] ---[ end trace 0000000000000000 ]---
[  792.411118][T74835] zram: Page 459 read from zram without previous write
```
