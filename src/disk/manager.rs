use super::page::PageId;
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    path::Path,
};

pub struct DiskManager {
    heap_file: File,
    next_page_id: PageId,
}

impl DiskManager {
    pub const PAGE_SIZE: usize = 4096;

    pub fn new(heap_file: File) -> io::Result<Self> {
        let heap_file_size = heap_file.metadata()?.len();
        let next_page_id = PageId::new(heap_file_size / Self::PAGE_SIZE as u64);
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

    pub fn write_page_data(&mut self, page_id: PageId, data: &[u8]) -> io::Result<()> {
        let offset = Self::calc_offset(page_id);
        self.heap_file.seek(SeekFrom::Start(offset))?;
        self.heap_file.write_all(data)
    }

    pub fn read_page_data(&mut self, page_id: PageId, data: &mut [u8]) -> io::Result<()> {
        let offset = Self::calc_offset(page_id);
        self.heap_file.seek(SeekFrom::Start(offset))?;
        self.heap_file.read_exact(data)
    }

    pub fn sync(&mut self) -> io::Result<()> {
        self.heap_file.flush()?;
        self.heap_file.sync_all()
    }

    fn calc_offset(page_id: PageId) -> u64 {
        page_id.value() * Self::PAGE_SIZE as u64
    }
}

#[cfg(test)]
mod disk_manager_test {
    use super::*;

    mod new {
        use super::*;
        use std::fs::remove_file;

        #[allow(non_snake_case)]
        #[test]
        fn DiskManagerが正しく生成されること() {
            // Arrange
            let file_path = "disk_manager_test::new::0.txt";
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
            assert_eq!(disk.next_page_id, PageId::new(0));

            // Cleanup
            remove_file(file_path).unwrap();
        }
    }

    mod open {
        use super::*;
        use std::fs::remove_file;

        #[test]
        fn すでに存在するファイルを正しく開けること() {
            // Arrange
            let file_path = "disk_manager_test::open::0.txt";
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
            assert_eq!(disk.next_page_id, PageId::new(0));

            // Cleanup
            remove_file(file_path).unwrap();
        }
    }

    mod allocate_page {
        use super::*;
        use std::fs::remove_file;

        #[allow(non_snake_case)]
        #[test]
        fn 現在のページIDを返し内部の値はインクリメントされていること() {
            // Arrange
            let file_path = "disk_manager_test::allocate_page::0.txt";
            let mut file = File::create(file_path).unwrap();
            file.flush().unwrap();

            // Act
            let mut disk = DiskManager::new(file).unwrap();
            let page_id = disk.allocate_page();

            // Assert
            assert_eq!(page_id, PageId::new(0));
            assert_eq!(disk.next_page_id, PageId::new(0).next());

            // Cleanup
            remove_file(file_path).unwrap();
        }
    }

    mod write_page_data {
        use super::*;
        use std::fs::remove_file;

        #[test]
        fn データをファイルに書き込めること() {
            // Arrange
            let file_path = "disk_manager_test::write_page_data::0.txt";
            let mut disk = DiskManager::open(file_path).unwrap();

            // Act
            disk.write_page_data(disk.next_page_id, b"Hello, world!")
                .unwrap();
            disk.heap_file.seek(SeekFrom::Start(0)).unwrap();
            let mut content = String::new();
            disk.heap_file.read_to_string(&mut content).unwrap();

            // Assert
            assert_eq!(content, "Hello, world!");

            // Cleanup
            remove_file(file_path).unwrap();
        }
    }

    mod read_page_data {
        use super::*;
        use std::fs::remove_file;

        #[test]
        fn ファイルに書き込まれたデータを読み込めること() {
            // Arrange
            let file_path = "disk_manager_test::read_page_data::0.txt";
            let mut disk = DiskManager::open(file_path).unwrap();
            disk.heap_file.write_all(b"Hello, world!").unwrap();

            // Act
            let mut data = vec![0u8; 13];
            disk.read_page_data(PageId::new(0), &mut data).unwrap();

            // Assert
            assert_eq!(data, b"Hello, world!");

            // Cleanup
            remove_file(file_path).unwrap();
        }
    }
}
