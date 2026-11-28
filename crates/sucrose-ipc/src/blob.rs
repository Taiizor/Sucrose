//! Payloads bigger than one frame: the service cuts them into [`ToRenderer::Blob`] chunks and the renderer puts them back
//! together (Pitfall 7). A frame stays at [`crate::frame::MAX_FRAME`]; a payload of up to [`BLOB_MAX`] bytes travels as at most
//! [`BLOB_MAX`] / [`BLOB_CHUNK`] chunks, in order, through the same writer queue as every other message.
//!
//! The first user is the album art of the Audio group (WAPI-06). The reader is bounded: one blob open at a time, a size
//! cap, a time limit, and any chunk that does not fit drops the blob (T-04-65).

use std::time::{Duration, Instant};

use crate::{portal::PortalEvent, protocol::ToRenderer};

/// The most bytes one chunk carries: a chunk plus its header stays well under a frame.
pub const BLOB_CHUNK: usize = 48 * 1024;
/// The most bytes a reassembled blob may have.
pub const BLOB_MAX: usize = 2 * 1024 * 1024;
/// How long a blob may take from its first chunk to its last.
pub const BLOB_WINDOW: Duration = Duration::from_secs(5);

/// Cuts `bytes` into the chunks of blob `id`: `seq` runs from 0 and `total` is the chunk count. An empty payload is one chunk
/// without bytes. A payload over [`BLOB_MAX`] is not cut: the result is empty, because a reader would refuse it anyway.
#[must_use]
pub fn chunks(id: u32, bytes: &[u8]) -> Vec<ToRenderer> {
    if bytes.len() > BLOB_MAX {
        return Vec::new();
    }
    let parts: Vec<&[u8]> = if bytes.is_empty() {
        vec![bytes]
    } else {
        bytes.chunks(BLOB_CHUNK).collect()
    };
    // At most 43 parts, so the count fits a u16.
    let total = u16::try_from(parts.len()).unwrap_or(u16::MAX);
    parts
        .into_iter()
        .zip(0_u16..)
        .map(|(part, seq)| ToRenderer::Blob {
            id,
            seq,
            total,
            bytes: part.to_vec(),
        })
        .collect()
}

/// Cuts `bytes` into the [`PortalEvent::PropertiesBlob`] chunks of document `id`, with the same rules as [`chunks`]: `seq`
/// runs from 0, an empty document is one empty chunk and one over [`BLOB_MAX`] gives no chunks.
#[must_use]
pub fn property_chunks(id: u32, bytes: &[u8]) -> Vec<PortalEvent> {
    chunks(id, bytes)
        .into_iter()
        .filter_map(|message| match message {
            ToRenderer::Blob { id, seq, total, bytes } => Some(PortalEvent::PropertiesBlob { id, seq, total, bytes }),
            _ => None,
        })
        .collect()
}

/// Why a blob was dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropReason {
    /// The first chunk seen of a blob was not chunk 0, or it announced no chunks.
    BadStart,
    /// A chunk arrived out of order.
    OutOfOrder,
    /// A chunk announced a different chunk count than the first one.
    TotalChanged,
    /// A chunk was bigger than [`BLOB_CHUNK`], or the blob grew past [`BLOB_MAX`].
    TooLarge,
    /// The blob took longer than [`BLOB_WINDOW`].
    TooSlow,
}

/// What a chunk did to the reassembly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Stored; more chunks are expected.
    Pending,
    /// The last chunk arrived: the blob's id and bytes.
    Done(u32, Vec<u8>),
    /// The blob was dropped now, for this reason (reported once per blob).
    Dropped(u32, DropReason),
    /// A chunk of a blob that was dropped earlier: nothing happens, nothing to report again.
    Ignored,
}

struct Open {
    id: u32,
    total: u16,
    next: u16,
    bytes: Vec<u8>,
    started: Instant,
}

/// Puts the chunks of one blob at a time back together.
#[derive(Default)]
pub struct Reassembler {
    open: Option<Open>,
    /// The blob that was dropped last, so its remaining chunks are ignored without a second report.
    dropped: Option<u32>,
}

impl Reassembler {
    /// A reader with no blob open.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Takes one chunk, timed by the clock.
    pub fn push(&mut self, id: u32, seq: u16, total: u16, bytes: &[u8]) -> Outcome {
        self.push_at(Instant::now(), id, seq, total, bytes)
    }

    /// Takes one chunk that arrived at `now`. A chunk with a new id while another blob is open restarts with the new id.
    pub fn push_at(&mut self, now: Instant, id: u32, seq: u16, total: u16, bytes: &[u8]) -> Outcome {
        if self.open.as_ref().is_some_and(|open| open.id != id) {
            self.open = None;
        }
        if self.open.is_none() {
            if self.dropped == Some(id) {
                return Outcome::Ignored;
            }
            if seq != 0 || total == 0 {
                return self.drop_blob(id, DropReason::BadStart);
            }
            self.dropped = None;
            self.open = Some(Open {
                id,
                total,
                next: 0,
                bytes: Vec::new(),
                started: now,
            });
        }
        let Some(open) = self.open.as_mut() else {
            return Outcome::Ignored;
        };
        let reason = if seq != open.next {
            Some(DropReason::OutOfOrder)
        } else if total != open.total {
            Some(DropReason::TotalChanged)
        } else if bytes.len() > BLOB_CHUNK || open.bytes.len() + bytes.len() > BLOB_MAX {
            Some(DropReason::TooLarge)
        } else if now.saturating_duration_since(open.started) > BLOB_WINDOW {
            Some(DropReason::TooSlow)
        } else {
            None
        };
        if let Some(reason) = reason {
            return self.drop_blob(id, reason);
        }
        open.bytes.extend_from_slice(bytes);
        open.next += 1;
        if open.next == open.total {
            let finished = self.open.take();
            return finished.map_or(Outcome::Ignored, |open| Outcome::Done(open.id, open.bytes));
        }
        Outcome::Pending
    }

    fn drop_blob(&mut self, id: u32, reason: DropReason) -> Outcome {
        self.open = None;
        self.dropped = Some(id);
        Outcome::Dropped(id, reason)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::MAX_FRAME;

    fn parts(messages: &[ToRenderer]) -> Vec<(u32, u16, u16, &[u8])> {
        messages
            .iter()
            .map(|message| match message {
                ToRenderer::Blob { id, seq, total, bytes } => (*id, *seq, *total, bytes.as_slice()),
                other => panic!("not a blob chunk: {other:?}"),
            })
            .collect()
    }

    fn feed(reader: &mut Reassembler, messages: &[ToRenderer]) -> Vec<Outcome> {
        parts(messages)
            .into_iter()
            .map(|(id, seq, total, bytes)| reader.push(id, seq, total, bytes))
            .collect()
    }

    #[test]
    fn a_payload_of_120_kib_is_three_chunks_of_at_most_the_chunk_size() {
        let payload: Vec<u8> = (0..120 * 1024).map(|n| (n % 251) as u8).collect();

        let messages = chunks(7, &payload);

        let parts = parts(&messages);
        assert_eq!(parts.len(), 3);
        for (index, (id, seq, total, bytes)) in parts.iter().enumerate() {
            assert_eq!((*id, usize::from(*seq), *total), (7, index, 3));
            assert!(bytes.len() <= BLOB_CHUNK);
        }
        let joined: Vec<u8> = parts.iter().flat_map(|part| part.3.iter().copied()).collect();
        assert_eq!(joined, payload);
    }

    #[test]
    fn a_document_of_the_largest_size_round_trips_through_property_chunks() {
        let document: Vec<u8> = (0..BLOB_MAX).map(|n| (n % 241) as u8).collect();
        let mut reader = Reassembler::new();
        let mut done = None;

        for event in property_chunks(3, &document) {
            let PortalEvent::PropertiesBlob { id, seq, total, bytes } = event else {
                panic!("not a properties chunk");
            };
            assert!(bytes.len() <= BLOB_CHUNK);
            if let Outcome::Done(id, bytes) = reader.push(id, seq, total, &bytes) {
                done = Some((id, bytes));
            }
        }

        assert_eq!(done, Some((3, document)));
    }

    #[test]
    fn a_document_over_the_limit_gives_no_property_chunks() {
        assert!(property_chunks(1, &vec![0; BLOB_MAX + 1]).is_empty());
    }

    #[test]
    fn the_last_chunk_gives_the_bytes_back() {
        let payload: Vec<u8> = (0..120 * 1024).map(|n| (n % 251) as u8).collect();
        let mut reader = Reassembler::new();

        let outcomes = feed(&mut reader, &chunks(9, &payload));

        assert_eq!(outcomes[0], Outcome::Pending);
        assert_eq!(outcomes[1], Outcome::Pending);
        assert_eq!(outcomes[2], Outcome::Done(9, payload));
    }

    #[test]
    fn an_empty_payload_is_one_chunk_without_bytes() {
        let messages = chunks(1, &[]);

        assert_eq!(parts(&messages), vec![(1, 0, 1, &[][..])]);
        assert_eq!(
            feed(&mut Reassembler::new(), &messages),
            vec![Outcome::Done(1, Vec::new())]
        );
    }

    #[test]
    fn a_payload_over_the_cap_is_not_cut() {
        assert!(chunks(1, &vec![0; BLOB_MAX + 1]).is_empty());
        assert!(!chunks(1, &vec![0; BLOB_MAX]).is_empty());
    }

    #[test]
    fn a_full_chunk_fits_one_frame() {
        let messages = chunks(u32::MAX, &vec![0xAB; BLOB_CHUNK * 2]);

        for message in &messages {
            let bytes = postcard::to_stdvec(message).expect("encode");
            assert!(bytes.len() < MAX_FRAME as usize, "{}", bytes.len());
        }
    }

    #[test]
    fn a_new_id_while_a_blob_is_open_restarts_with_the_new_id() {
        let mut reader = Reassembler::new();
        let first = chunks(1, &vec![1; BLOB_CHUNK * 2]);
        let second = chunks(2, &[5, 6, 7]);
        let (a, b) = (parts(&first), parts(&second));

        assert_eq!(reader.push(a[0].0, a[0].1, a[0].2, a[0].3), Outcome::Pending);
        assert_eq!(
            reader.push(b[0].0, b[0].1, b[0].2, b[0].3),
            Outcome::Done(2, vec![5, 6, 7])
        );
        assert_eq!(
            reader.push(a[1].0, a[1].1, a[1].2, a[1].3),
            Outcome::Dropped(1, DropReason::BadStart),
            "the second chunk of the abandoned blob has no start"
        );
    }

    #[test]
    fn a_chunk_out_of_order_drops_the_blob_once() {
        let mut reader = Reassembler::new();
        let messages = chunks(3, &vec![1; BLOB_CHUNK * 3]);
        let p = parts(&messages);

        assert_eq!(reader.push(p[0].0, p[0].1, p[0].2, p[0].3), Outcome::Pending);
        assert_eq!(
            reader.push(p[2].0, p[2].1, p[2].2, p[2].3),
            Outcome::Dropped(3, DropReason::OutOfOrder)
        );
        assert_eq!(reader.push(p[1].0, p[1].1, p[1].2, p[1].3), Outcome::Ignored);
        assert_eq!(reader.push(p[2].0, p[2].1, p[2].2, p[2].3), Outcome::Ignored);
    }

    #[test]
    fn a_blob_that_does_not_start_at_chunk_zero_or_announces_nothing_is_dropped() {
        let mut reader = Reassembler::new();

        assert_eq!(reader.push(1, 1, 2, &[1]), Outcome::Dropped(1, DropReason::BadStart));
        assert_eq!(reader.push(2, 0, 0, &[]), Outcome::Dropped(2, DropReason::BadStart));
    }

    #[test]
    fn a_changed_chunk_count_drops_the_blob() {
        let mut reader = Reassembler::new();

        assert_eq!(reader.push(1, 0, 3, &[1]), Outcome::Pending);
        assert_eq!(
            reader.push(1, 1, 4, &[1]),
            Outcome::Dropped(1, DropReason::TotalChanged)
        );
    }

    #[test]
    fn a_blob_over_the_cap_or_a_chunk_over_the_chunk_size_is_dropped() {
        let mut reader = Reassembler::new();
        let big = vec![0_u8; BLOB_CHUNK];
        let total = u16::try_from(BLOB_MAX / BLOB_CHUNK + 2).expect("fits");

        let mut last = Outcome::Pending;
        for seq in 0..total {
            last = reader.push(1, seq, total, &big);
            if last != Outcome::Pending {
                break;
            }
        }
        assert_eq!(last, Outcome::Dropped(1, DropReason::TooLarge));
        assert_eq!(
            Reassembler::new().push(2, 0, 1, &vec![0; BLOB_CHUNK + 1]),
            Outcome::Dropped(2, DropReason::TooLarge)
        );
    }

    #[test]
    fn a_blob_that_takes_more_than_five_seconds_is_dropped() {
        let mut reader = Reassembler::new();
        let start = Instant::now();

        assert_eq!(reader.push_at(start, 1, 0, 2, &[1]), Outcome::Pending);
        assert_eq!(
            reader.push_at(start + BLOB_WINDOW + Duration::from_millis(1), 1, 1, 2, &[1]),
            Outcome::Dropped(1, DropReason::TooSlow)
        );
        let mut quick = Reassembler::new();
        assert_eq!(quick.push_at(start, 1, 0, 2, &[1]), Outcome::Pending);
        assert_eq!(
            quick.push_at(start + BLOB_WINDOW, 1, 1, 2, &[2]),
            Outcome::Done(1, vec![1, 2])
        );
    }

    #[test]
    fn a_blob_after_a_dropped_one_is_read_normally() {
        let mut reader = Reassembler::new();
        assert_eq!(reader.push(1, 1, 2, &[1]), Outcome::Dropped(1, DropReason::BadStart));

        let outcomes = feed(&mut reader, &chunks(2, &[9; 10]));

        assert_eq!(outcomes, vec![Outcome::Done(2, vec![9; 10])]);
    }
}
