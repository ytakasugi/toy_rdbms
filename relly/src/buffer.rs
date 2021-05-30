use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::io;
use std::ops::{Index, IndexMut};
use std::rc::Rc;

use crate::disk::{DiskManager, PageId, PAGE_SIZE};

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub struct BufferId(usize);

// `u8`型の配列
pub type Page = [u8; PAGE_SIZE];

#[derive(Debug)]
pub struct Buffer {
    pub page_id: PageId,
    pub page: RefCell<Page>,
    pub is_dirty: Cell<bool>,
}

#[derive(Debug, Default)]
pub struct Frame {
    // Bufferの利用回数
    usage_count: u64,
    buffer: Rc<Buffer>,
}

pub struct BufferPool {
    buffers: Vec<Frame>,
    next_victim_id: BufferId,
}

impl BufferPool {
    pub fn new(pool_size: usize) -> Self {
        let mut buffers = vec![];
        buffers.resize_with(pool_size, Default::default);
        let next_victim_id = BufferId::default();
        Self {
            buffers,
            next_victim_id,
        }
    }

    // BufferPoolのサイズを取得するメソッド
    fn size(&self) -> usize {
        self.buffers.len()
    }

    fn evict(&mut self) -> Option<BufferId> {
        // BufferPoolサイズの取得
        let pool_size = self.size();
        // `consecutive_pinned`カウンタを初期化
        let mut consecutive_pinned = 0;
        // BufferPool内をループし、破棄するBufferを決定する
        let victim_id = loop {
            // このloop内での`next_victim_id`は、`BufferPool::next_victim_id`に束縛する
            let next_victim_id = self.next_victim_id;
            let frame = &mut self[next_victim_id];
            // もし、Bufferの利用回数が0だった場合・・・
            if frame.usage_count == 0 {
                // `next_victim_id`を返す
                break self.next_victim_id;
            }
            // もし、Bufferを貸出中でない場合は、Bufferの`usage_count`をデクリメントする
            if Rc::get_mut(&mut frame.buffer).is_some() {
                frame.usage_count -= 1;
                consecutive_pinned = 0;
            // 貸出中の場合、`consecutive_pinned`カウンタをインクリメントする
            } else {
                consecutive_pinned += 1;
                // もし、`consecutive_pinned`カウンタがBufferPool以上であった場合は、`None`を返す
                if consecutive_pinned >= pool_size {
                    return None;
                }
            }
            self.next_victim_id = self.increment_id(self.next_victim_id);
        };
        Some(victim_id)
    }

    fn increment_id(&self, buffer_id: BufferId) -> BufferId {
        // `BufferId`に1を足して、要素数で割ったあまりを返す
        BufferId((buffer_id.0 + 1) % self.size())
    }
}

pub struct BufferPoolManager {
    disk: DiskManager,
    pool: BufferPool,
    page_table: HashMap<PageId, BufferId>,
}