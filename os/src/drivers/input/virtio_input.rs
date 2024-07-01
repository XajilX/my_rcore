use alloc::collections::VecDeque;
use virtio_drivers::{device::input::VirtIOInput, transport::mmio::MmioTransport};

use crate::{drivers::{Device, VirtioHal}, sync::{CondVar, UThrCell}, task::processor::schedule};

use super::InputDevice;

pub struct VirtioInput {
    inner: UThrCell<VirtioInputInner>,
    condvar: CondVar
}

struct VirtioInputInner {
    virtio_input: VirtIOInput<VirtioHal, MmioTransport>,
    events: VecDeque<u64>
}

impl VirtioInput {
    pub fn new(transport: MmioTransport) -> Self {
        let inner = VirtioInputInner {
            virtio_input: VirtIOInput::<VirtioHal, MmioTransport>::new(
                transport
            ).unwrap(),
            events: VecDeque::new()
        };
        Self {
            inner: unsafe { UThrCell::new(inner) },
            condvar: CondVar::new()
        }
    }
}

impl InputDevice for VirtioInput {
    fn read_event(&self) -> u64 {
        loop {
            let mut inner = self.inner.get_refmut();
            if let Some(event) = inner.events.pop_front() {
                return event;
            } else {
                let task_cx = self.condvar.wait_without_schd();
                drop(inner);
                schedule(task_cx);
            }
        }
    }
    fn is_empty(&self) -> bool {
        self.inner.get_refmut().events.is_empty()
    }
}

impl Device for VirtioInput {
    fn handle_irq(&self) {
        let mut cnt = 0;
        let mut res = 0;
        self.inner.then(|inner| {
            inner.virtio_input.ack_interrupt();
            while let Some(event) = inner.virtio_input.pop_pending_event() {
                cnt += 1;
                res = (event.event_type as u64) << 48 |
                    (event.code as u64) << 32 |
                    (event.value as u64);
            }
        });
        if cnt > 0 {
            self.condvar.signal();
        };
    }
}