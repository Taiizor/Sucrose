//! Process memory readings that separate the candidates for a growth in private bytes: `heap_allocated` is what Rust code
//! keeps alive (decode buffers, caches); `heap_committed` minus `heap_allocated` is what the heap keeps after frees; the
//! video memory of this process is where GPUI's atlas textures are counted. Rust's `System` allocator on Windows
//! allocates from `GetProcessHeap()`, and GPUI declares no global allocator outside its benches, so the process heap is
//! the heap the Portal's Rust code uses.

use windows::{
    Win32::{
        Graphics::Dxgi::{
            CreateDXGIFactory1, DXGI_ERROR_NOT_FOUND, DXGI_MEMORY_SEGMENT_GROUP, DXGI_MEMORY_SEGMENT_GROUP_LOCAL,
            DXGI_MEMORY_SEGMENT_GROUP_NON_LOCAL, DXGI_QUERY_VIDEO_MEMORY_INFO, IDXGIAdapter3, IDXGIFactory1,
        },
        System::Memory::{GetProcessHeap, HEAP_SUMMARY, HeapSummary},
    },
    core::Interface,
};

use crate::MemoryDetail;

/// The heap and video memory readings of this process; a reading the OS refuses is `None`.
pub(crate) fn memory_detail() -> MemoryDetail {
    let heap = heap_bytes();
    let video = video_memory_bytes();
    MemoryDetail {
        heap_allocated: heap.map(|(allocated, _)| allocated),
        heap_committed: heap.map(|(_, committed)| committed),
        gpu_local: video.map(|(local, _)| local),
        gpu_non_local: video.map(|(_, non_local)| non_local),
    }
}

/// Allocated and committed bytes of the process heap.
fn heap_bytes() -> Option<(u64, u64)> {
    // SAFETY: GetProcessHeap has no preconditions; the result is checked.
    let heap = match unsafe { GetProcessHeap() } {
        Ok(heap) => heap,
        Err(error) => {
            tracing::debug!(error = %error, "GetProcessHeap failed, no heap reading");
            return None;
        }
    };
    let mut summary = HEAP_SUMMARY {
        cb: u32::try_from(size_of::<HEAP_SUMMARY>()).ok()?,
        ..HEAP_SUMMARY::default()
    };
    // SAFETY: `heap` is the live process heap handle and `summary` is a writable HEAP_SUMMARY whose `cb` holds its
    // size, as HeapSummary requires; flags 0 is the only documented value.
    if let Err(error) = unsafe { HeapSummary(heap, 0, &raw mut summary) }.ok() {
        tracing::debug!(error = %error, "HeapSummary failed, no heap reading");
        return None;
    }
    Some((
        u64::try_from(summary.cbAllocated).ok()?,
        u64::try_from(summary.cbCommitted).ok()?,
    ))
}

/// This process's video memory in use on the local and the non-local segment group, summed over every adapter that
/// answers. `None` when no adapter does.
fn video_memory_bytes() -> Option<(u64, u64)> {
    // SAFETY: CreateDXGIFactory1 has no preconditions; the result is checked.
    let factory = match unsafe { CreateDXGIFactory1::<IDXGIFactory1>() } {
        Ok(factory) => factory,
        Err(error) => {
            tracing::debug!(error = %error, "CreateDXGIFactory1 failed, no video memory reading");
            return None;
        }
    };
    let mut totals: Option<(u64, u64)> = None;
    for index in 0.. {
        // SAFETY: `factory` is a live COM interface; running past the last adapter is reported as an error.
        let adapter = match unsafe { factory.EnumAdapters1(index) } {
            Ok(adapter) => adapter,
            Err(error) if error.code() == DXGI_ERROR_NOT_FOUND => break,
            Err(error) => {
                tracing::debug!(error = %error, index, "EnumAdapters1 failed, adapter list ends here");
                break;
            }
        };
        let Ok(adapter) = adapter.cast::<IDXGIAdapter3>() else {
            tracing::debug!(index, "adapter has no IDXGIAdapter3, skipped");
            continue;
        };
        let (Some(local), Some(non_local)) = (
            current_usage(&adapter, DXGI_MEMORY_SEGMENT_GROUP_LOCAL),
            current_usage(&adapter, DXGI_MEMORY_SEGMENT_GROUP_NON_LOCAL),
        ) else {
            continue;
        };
        let (total_local, total_non_local) = totals.unwrap_or((0, 0));
        totals = Some((
            total_local.saturating_add(local),
            total_non_local.saturating_add(non_local),
        ));
    }
    totals
}

/// The bytes this process uses in one segment group of `adapter` (node 0).
fn current_usage(adapter: &IDXGIAdapter3, group: DXGI_MEMORY_SEGMENT_GROUP) -> Option<u64> {
    let mut info = DXGI_QUERY_VIDEO_MEMORY_INFO::default();
    // SAFETY: `adapter` is a live COM interface and `info` is a writable DXGI_QUERY_VIDEO_MEMORY_INFO; node 0 always
    // exists.
    match unsafe { adapter.QueryVideoMemoryInfo(0, group, &raw mut info) } {
        Ok(()) => Some(info.CurrentUsage),
        Err(error) => {
            tracing::debug!(error = %error, "QueryVideoMemoryInfo failed, adapter skipped");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hint::black_box;

    const MIB: u64 = 1024 * 1024;

    #[test]
    #[allow(clippy::print_stderr)]
    fn memory_detail_counts_live_heap_blocks() {
        // One large block, taken in this test so no parallel test disturbs the readings: does a decode-sized buffer show?
        let base = memory_detail().heap_allocated.unwrap_or(0);
        let large = vec![1_u8; 64 * 1024 * 1024];
        let with_large = memory_detail().heap_allocated.unwrap_or(0);
        black_box(&large);
        drop(large);
        eprintln!(
            "one live 64 MiB Vec raised heap_allocated by {} MiB",
            with_large.saturating_sub(base) / MIB
        );
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
