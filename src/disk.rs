use std::path::Path;
use std::{
    fs::{File, OpenOptions},
    io,
};

pub const PAGE_SIZE: usize = 4096;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PageId(u64);
impl PageId {
    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod PageIdTests {
    use super::*;

    mod next {
        use super::*;

        #[test]
        fn _1だけ足されたPageIdを返すこと() {
            let page_id = PageId(0).next();
            assert_eq!(page_id, PageId(1));
        }
    }
}

pub struct DiskManager {
    heap_file: File,
    next_page_id: PageId,
}

impl DiskManager {
    pub fn new(heap_file: File) -> io::Result<Self> {
        let heap_file_size = heap_file.metadata()?.len();
        let next_page_id = PageId(heap_file_size / PAGE_SIZE as u64);
        Ok(Self {
            heap_file,
            next_page_id,
        })
    }

    pub fn open(heap_file_path: impl AsRef<Path>) -> io::Result<Self> {
        let heap_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(heap_file_path)?;
        Self::new(heap_file)
    }

    pub fn allocate_page(&mut self) -> PageId {
        let page_id = self.next_page_id;
        self.next_page_id = page_id.next();
        page_id
    }

    // pub fn read_page_data(&mut self, page_id: PageId, data: &mut [u8]) -> io::Result<()> {}

    // pub fn write_page_data(&mut self, page_id: PageId, data: &[u8]) -> io::Result<()> {}
}

#[cfg(test)]
#[allow(non_snake_case)]
mod DiskManagerTests {
    use super::*;

    mod new {
        use super::*;
        use std::{
            fs::{remove_file, OpenOptions},
            io::{Read, Seek, SeekFrom, Write},
        };

        #[test]
        fn DiskManagerが正しく生成されること() {
            // Arrange
            let file_path = "new_0.txt";
            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(file_path)
                .unwrap();
            file.write_all(b"Hello, world!").unwrap();
            file.seek(SeekFrom::Start(0)).unwrap();

            // Act
            let mut disk = DiskManager::new(file).unwrap();
            let mut content = String::new();
            disk.heap_file.read_to_string(&mut content).unwrap();

            // Assert
            assert_eq!(content, "Hello, world!");
            assert_eq!(disk.next_page_id, PageId(0));

            // Cleanup
            remove_file(file_path).unwrap();
        }
    }

    mod open {
        use super::*;
        use std::{
            fs::{remove_file, OpenOptions},
            io::{Read, Write},
        };

        #[test]
        fn すでに存在するファイルを正しく開けること() {
            // Arrange
            let file_path = "open_0.txt";
            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(file_path)
                .unwrap();
            file.write_all(b"Hello, world!").unwrap();
            file.flush().unwrap();

            // Act
            let mut disk = DiskManager::open(file_path).unwrap();
            let mut content = String::new();
            disk.heap_file.read_to_string(&mut content).unwrap();

            // Assert
            assert_eq!(content, "Hello, world!");
            assert_eq!(disk.next_page_id, PageId(0));

            // Cleanup
            remove_file(file_path).unwrap();
        }
    }

    mod allocate_page {
        use super::*;
        use std::{fs::remove_file, io::Write};

        #[test]
        fn 現在のページIDを返し内部の値はインクリメントされていること() {
            // Arrange
            let file_path = "allocate_page_0.txt";
            let mut file = File::create(file_path).unwrap();
            file.flush().unwrap();

            // Act
            let mut disk = DiskManager::new(file).unwrap();
            let page_id = disk.allocate_page();

            // Assert
            assert_eq!(page_id, PageId(0));
            assert_eq!(disk.next_page_id, PageId(0).next());

            // Cleanup
            remove_file(file_path).unwrap();
        }
    }
}
