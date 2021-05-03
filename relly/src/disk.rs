use std::convert::TryInto;
use std::fs::{File, OpenOptions};
use std::io::{self, prelude::*, SeekFrom};
use std::path::Path;

use zerocopy::{AsBytes, FromBytes};

pub const PAGE_SIZE: usize = 4096;

pub struct PageId(pub u64);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, FromBytes, AsBytes)]
#[repr(C)]
impl PageId {
    // 無効なページID
    pub const INVALID_PAGE_ID: PageId = PageId(u64::MAX);

    pub fn valid(self) -> Option<PageId> {
        // もし、無効なページIDと等しい場合は、`None`
        if self == Self::INVALID_PAGE_ID {
            None
        // そうでなければ、`self`を`Some`でラップ
        } else {
            Some(self)
        }
    }

    // u64型へ変換
    pub fn to_u64(self) -> u64 {
        self.0
    }
}

// `PageId`に`Default`トレイトを実装
impl Default for PageId {
    fn default() -> Self {
        Self::INVALID_PAGE_ID
    }
}

// `From<変換元> for `変換先`
// `Option<PageId>から`PageId`への変換方法を定義するために、`PageId`に`From<Option<PageId>>`を実装
impl From<Option<PageId>> for PageId {
    fn from(page_id: Option<PageId>) -> Self {
        page_id.unwrap_or_default()
    }
}

// `From<変換元> for `変換先`
// `&[u8]`から`PageId`への変換方法を定義するために、`PageId`に`From<&[u8]>`を実装
impl From<&[u8]> for PageId {
    fn from(bytes: &[u8]) -> Self {
        let arr = bytes.try_into().unwrap();
        PageId(u64::from_ne_bytes(arr))
    }
}

pub struct DiskManager {
    // ヒープファイルのファイルディスクリプタ
    heap_file: File,
    // 採番するページIDを決めるカウンタ
    next_page_id: u64,
}

impl DiskManager {
    pub fn new(heap_file: File) -> io::Result<Self> {
        // ヒープファイルのサイズを取得
        // ヒープファイルのメタデータを取得して、そのサイズをバイト数で返す。
        // メタデータ取得に失敗した場合、エラーが返却された場合、早期returnする。
        let heap_file_size = heap_file.metadata()?.len();
        // 採番するページIDを算出する
        let next_page_id = heap_file / PAGESIZE as u64;
        Ok(Self {
            heap_file,
            next_page_id,
        }) 
    }

    // `AsRef<Path>`は、`&Path`に変換できるすべての参照を引数として扱うことができる
    pub fn open(heap_file: impl AsRef<Path>) -> io::Result<Self> {
        // 新しいOptionセットを作成
        let heap_file = OpenOptions::new()
                        // 読み込みアクセスの設定
                        .read(true)
                        // 書き込みアクセスの設定
                        .write(true)
                        // 新しいファイルを作成するか、既に存在している場合はそれを開くかのオプションを設定
                        .create(true)
                        // 指定されたパスのファイルを開く
                        .open(heap_file_path)?;
        Self::new(heap_file)
    }

    pub fn read_page_data(&mut self, page_id: PageId, data: &mut[u8]) -> io::Result<()> {
        // オフセットを計算
        let offset = PAGE_SIZE as u64 *  page_id.to_u64();
        // ページ先頭へシーク
        // `SeekFrom::Start`でファイルの先頭から`offset`バイト目を設定し、`seek`でシークする
        self.heap_file.seek(SeekFrom::Start(offset))?;
        // データを読み出す
        self.heap_file.read_exact(data)
    }

    pub fn write_page_data(&mut self, page_id: PageId, data: &[u8]) -> io::Result<()> {
        let offset = PAGE_SIZE as u64 * page_id.to_u64();
        self.heap_file.seek(SeekFrom::Start(offset))?;
        self.heap_file.write_all(data)
    }

    // 新しいページを作成するメソッド
    fn allocate_page(&mut self) -> PageId {
        let page_id = self.next_page_id;
        self.next_page_id += 1;
        PageId(page_id)
    }

}

