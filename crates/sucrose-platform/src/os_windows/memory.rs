//! Process memory readings that separate the candidates for a growth in private bytes: `heap_allocated` is what Rust code
//! keeps alive (decode buffers, caches); `heap_committed` minus `heap_allocated` is what the heap keeps after frees; the
//! video memory of this process is where GPUI's atlas textures are counted. Rust's `System` allocator on Windows
//! allocates from `GetProcessHeap()`, and GPUI declares no global allocator outside its benches, so the process heap is
//! the heap the Portal's Rust code uses.

use crate::MemoryDetail;

/// The heap and video memory readings of this process; a reading the OS refuses is `None`.
pub(crate) fn memory_detail() -> MemoryDetail {
    MemoryDetail::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hint::black_box;

    const MIB: u64 = 1024 * 1024;

    #[test]
    fn memory_detail_counts_live_heap_blocks() {
        let before = memory_detail();
        let blocks: Vec<Vec<u8>> = (0..16_384).map(|_| vec![1_u8; 4096]).collect();
        let alive = memory_detail();
        black_box(&blocks);
        drop(blocks);
        let after = memory_detail();
        let (before_a, alive_a, after_a) = (
            before.heap_allocated.expect("heap reading before"),
            alive.heap_allocated.expect("heap reading while alive"),
            after.heap_allocated.expect("heap reading after"),
        );
        assert!(alive_a >= before_a + 56 * MIB, "alive {alive_a} before {before_a}");
        assert!(after_a + 56 * MIB <= alive_a, "after {after_a} alive {alive_a}");
        for reading in [before, alive, after] {
            assert!(reading.heap_committed >= reading.heap_allocated);
        }
    }

    #[test]
    #[allow(clippy::print_stderr)]
    fn memory_detail_single_large_block_observation() {
        let before = memory_detail().heap_allocated.unwrap_or(0);
        let block = vec![1_u8; 64 * 1024 * 1024];
        let alive = memory_detail().heap_allocated.unwrap_or(0);
        black_box(&block);
        drop(block);
        eprintln!(
            "one live 64 MiB Vec raised heap_allocated by {} MiB",
            alive.saturating_sub(before) / MIB
        );
    }

    #[test]
    fn memory_detail_reports_video_memory_when_an_adapter_exists() {
        let detail = memory_detail();
        if super::super::gpu::gpu_adapters().is_empty() {
            assert_eq!((detail.gpu_local, detail.gpu_non_local), (None, None));
        } else {
            assert!(
                detail.gpu_local.is_some() && detail.gpu_non_local.is_some(),
                "{detail:?}"
            );
        }
    }
}
