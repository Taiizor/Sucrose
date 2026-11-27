//! Mouse-move coalescing for the input a web engine forwards: a mouse moves a thousand times a second, so consecutive
//! moves share one slot and one queue marker instead of filling the command queue. Only a move that directly follows
//! another move is dropped; a click, key or wheel event keeps its place between the moves around it (TYPE-04).

use std::sync::{Mutex, PoisonError};

/// The newest mouse move not yet handed to the page, and the markers that stopped meaning anything.
///
/// The handle side calls [`offer`](Self::offer) for a move and [`flush_before_other`](Self::flush_before_other) for every
/// other event; the pump side calls [`take_at_marker`](Self::take_at_marker) for each marker it reaches in the queue.
/// The handle is driven from one thread at a time.
#[derive(Debug)]
pub struct MoveSlot<T> {
    inner: Mutex<Inner<T>>,
}

#[derive(Debug)]
struct Inner<T> {
    pending: Option<T>,
    /// Markers still in the queue whose move was queued ahead of a later event: the pump skips that many markers, in
    /// order, because the markers are taken first-in first-out and these are the oldest.
    orphans: usize,
}

impl<T> Default for MoveSlot<T> {
    fn default() -> Self {
        Self {
            inner: Mutex::new(Inner {
                pending: None,
                orphans: 0,
            }),
        }
    }
}

impl<T: Copy> MoveSlot<T> {
    fn lock(&self) -> std::sync::MutexGuard<'_, Inner<T>> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Keeps `event` as the newest move, replacing an older one that was not handed over yet. `true` when no marker
    /// was waiting for a move: the caller queues one.
    pub fn offer(&self, event: T) -> bool {
        self.lock().pending.replace(event).is_none()
    }

    /// The marker `offer` asked for could not be queued, so nobody will take the pending move: it is dropped and the
    /// next move asks for a marker again.
    pub fn marker_dropped(&self) {
        self.lock().pending = None;
    }

    /// Another kind of event is about to be queued. A move still waiting must reach the page before it, so it is
    /// returned for the caller to queue first; its marker is left behind and the pump skips it.
    pub fn flush_before_other(&self) -> Option<T> {
        let mut inner = self.lock();
        let moved = inner.pending.take();
        if moved.is_some() {
            inner.orphans += 1;
        }
        moved
    }

    /// The pump reached a marker: the move to hand to the page, or `None` when the marker was left behind by
    /// [`flush_before_other`](Self::flush_before_other).
    pub fn take_at_marker(&self) -> Option<T> {
        let mut inner = self.lock();
        if inner.orphans > 0 {
            inner.orphans -= 1;
            return None;
        }
        inner.pending.take()
    }

    /// The move waiting now, without taking it.
    pub fn pending(&self) -> Option<T> {
        self.lock().pending
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Event {
        Move(u32),
        Other(u32),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Queued {
        Marker,
        Input(Event),
    }

    /// The handle side of an engine over a queue of `capacity`.
    fn forward(slot: &MoveSlot<Event>, queue: &mut VecDeque<Queued>, capacity: usize, event: Event) {
        let mut send = |item| {
            if queue.len() < capacity {
                queue.push_back(item);
                true
            } else {
                false
            }
        };
        match event {
            Event::Move(_) => {
                if slot.offer(event) && !send(Queued::Marker) {
                    slot.marker_dropped();
                }
            }
            Event::Other(_) => {
                if let Some(moved) = slot.flush_before_other() {
                    send(Queued::Input(moved));
                }
                send(Queued::Input(event));
            }
        }
    }

    /// The pump side: what the page receives, in order.
    fn drain(slot: &MoveSlot<Event>, queue: &mut VecDeque<Queued>) -> Vec<Event> {
        let mut delivered = Vec::new();
        while let Some(item) = queue.pop_front() {
            match item {
                Queued::Marker => delivered.extend(slot.take_at_marker()),
                Queued::Input(event) => delivered.push(event),
            }
        }
        delivered
    }

    fn run(events: &[Event]) -> Vec<Event> {
        let slot = MoveSlot::default();
        let mut queue = VecDeque::new();
        for event in events {
            forward(&slot, &mut queue, 64, *event);
        }
        drain(&slot, &mut queue)
    }

    #[test]
    fn consecutive_moves_keep_only_the_newest() {
        assert_eq!(run(&[Event::Move(1), Event::Move(2), Event::Move(3)]), [Event::Move(3)]);
    }

    #[test]
    fn a_click_between_two_moves_keeps_its_place_and_both_moves() {
        assert_eq!(
            run(&[Event::Move(1), Event::Other(10), Event::Move(2)]),
            [Event::Move(1), Event::Other(10), Event::Move(2)]
        );
    }

    #[test]
    fn only_the_older_of_two_consecutive_moves_is_dropped_around_other_events() {
        assert_eq!(
            run(&[
                Event::Move(1),
                Event::Move(2),
                Event::Other(10),
                Event::Move(3),
                Event::Move(4),
                Event::Other(11),
                Event::Other(12),
                Event::Move(5),
            ]),
            [
                Event::Move(2),
                Event::Other(10),
                Event::Move(4),
                Event::Other(11),
                Event::Other(12),
                Event::Move(5)
            ]
        );
    }

    #[test]
    fn other_events_are_never_reordered_or_dropped_by_the_slot() {
        let events: Vec<Event> = (0..20).map(Event::Other).collect();
        assert_eq!(run(&events), events);
    }

    #[test]
    fn a_marker_that_could_not_be_queued_lets_the_next_move_ask_for_one_again() {
        let slot = MoveSlot::default();
        let mut queue = VecDeque::new();
        forward(&slot, &mut queue, 0, Event::Move(1));
        assert_eq!(slot.pending(), None);

        forward(&slot, &mut queue, 1, Event::Move(2));

        assert_eq!(queue, [Queued::Marker]);
        assert_eq!(drain(&slot, &mut queue), [Event::Move(2)]);
    }

    #[test]
    fn several_flushes_between_markers_skip_exactly_their_own_markers() {
        let slot = MoveSlot::default();
        let mut queue = VecDeque::new();
        for step in 0..5 {
            forward(&slot, &mut queue, 64, Event::Move(step));
            forward(&slot, &mut queue, 64, Event::Other(100 + step));
        }
        forward(&slot, &mut queue, 64, Event::Move(9));
        forward(&slot, &mut queue, 64, Event::Move(10));

        let delivered = drain(&slot, &mut queue);

        let mut expected: Vec<Event> = (0..5)
            .flat_map(|step| [Event::Move(step), Event::Other(100 + step)])
            .collect();
        expected.push(Event::Move(10));
        assert_eq!(delivered, expected);
    }
}
