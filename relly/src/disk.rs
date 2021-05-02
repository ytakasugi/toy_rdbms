use std::convert::TryInto;
use std::fs::{File, OpenOptions};
use std::io::{self, prelude::*, SeekFrom};
use std::path::Path;

use zerocopy::{AsBytes, FromBytes};

pub const PAGE_SIZE: usize = 4096;

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

}

