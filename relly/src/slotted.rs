// スロット：データベースの文脈における「スロット」は、
// データを格納するための領域を指す。
// 具体的には、データベースのページ（またはブロック）が複数のスロットに分割され、
// 各スロットが個々のレコードを格納する

use std::mem::size_of;
use std::ops::{Index, IndexMut, Range};

use zerocopy::{AsBytes, ByteSlice, ByteSliceMut, FromBytes, FromZeroes, Ref};

#[derive(Debug, AsBytes, FromBytes, FromZeroes)]
#[repr(C)]
pub struct Header {
    num_slots: u16,
    free_space_offset: u16,
    _pad: u32,
}

#[derive(Debug, AsBytes, FromBytes, FromZeroes, Clone, Copy)]
#[repr(C)]
pub struct Pointer {
    offset: u16,
    len: u16,
}

impl Pointer {
    fn range(&self) -> Range<usize> {
        let start = self.offset as usize;
        let end = start + self.len as usize;

        start..end
    }
}

pub type Pointers<B> = Ref<B, [Pointer]>;

pub struct Slotted<B> {
    header: Ref<B, Header>,
    body: B,
}
impl<B: ByteSlice> Slotted<B> {
    pub fn new(bytes: B) -> Self {
        let (header, body) = Ref::new_from_prefix(bytes).expect("slotted header must be aligned");

        Self { header, body }
    }

    pub fn capacity(&self) -> usize {
        self.body.len()
    }

    pub fn num_slots(&self) -> usize {
        self.header.num_slots as usize
    }

    pub fn free_space(&self) -> usize {
        self.header.free_space_offset as usize - self.pointers_size()
    }

    fn pointers_size(&self) -> usize {
        size_of::<Pointer>() * self.num_slots()
    }

    fn pointers(&self) -> Pointers<&[u8]> {
        Pointers::new_slice(&self.body[..self.pointers_size()]).unwrap()
    }

    fn data(&self, pointer: Pointer) -> &[u8] {
        &self.body[pointer.range()]
    }
}

impl<B: ByteSliceMut> Slotted<B> {
    pub fn initialize(&mut self) {
        self.header.num_slots = 0;
        self.header.free_space_offset = self.body.len() as u16;
    }

    fn pointers_mut(&mut self) -> Pointers<&mut [u8]> {
        let pointers_size = self.pointers_size();
        Pointers::new_slice(&mut self.body[..pointers_size]).unwrap()
    }

    fn data_mut(&mut self, pointer: Pointer) -> &mut [u8] {
        &mut self.body[pointer.range()]
    }

    pub fn insert(&mut self, index: usize, len: usize) -> Option<()> {
        // `self.free_space`がPointer型のインスタンスサイズと`len`の合計よりも小さい場合は、Noneを返す
        if self.free_space() < size_of::<Pointer>() + len {
            return None
        }

        let num_slots_org = self.num_slots();
        self.header.free_space_offset -= len as u16;
        self.header.num_slots += 1;
        let free_space_offset = self.header.free_space_offset;
        let mut pointers_mut = self.pointers_mut();
        pointers_mut.copy_within(index..num_slots_org, index + 1);
        let pointer = &mut pointers_mut[index];
        pointer.offset = free_space_offset;
        pointer.len = len as u16;

        Some(())
    }
}