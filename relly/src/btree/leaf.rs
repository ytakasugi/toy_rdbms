use std::mem::size_of;

use zerocopy::{AsBytes, ByteSlice, FromBytes, FromZeroes, Ref};

use super::Pair;
use crate::bsearch::binary_search_by;
use crate::buffer::Page;
use crate::disk::PageId;
use crate::slotted::{self, Slotted};

#[derive(Debug, FromBytes, FromZeroes, AsBytes)]
#[repr(C)]
pub struct Header {
    prev_page_id: PageId,
    next_page_id: PageId,
}

pub struct Leaf<B> {
    header: Ref<B, Header>,
    body: Slotted<B>,
}

impl<B: ByteSlice> Leaf<B> {
    pub fn new(bytes: B) -> Self {
        let (header, body) = 
            Ref::new_from_prefix(bytes).expect("leaf header must be aligned");
        let body = Slotted::new(body);
        Self { header, body }
    }

    pub fn prev_page_id(&self) -> Option<PageId> {
        self.header.prev_page_id.valid()
    }

    pub fn next_page_id(&self) -> Option<PageId> {
        self.header.next_page_id.valid()
    }

    pub fn num_pairs(&self) -> usize {
        self.body.num_slots()
    }

    pub fn search_slot_id(&self, key: &[u8]) -> Result<usize, usize> {
        binary_search_by(self.num_pairs(), |slot_id| {
            self.pair_at(slot_id).key.cmp(key)
        })
    }

    pub fn pair_at(&self, slot_id: usize) -> Pair {
        Pair::from_bytes(&self.body[slot_id])
    }
}
