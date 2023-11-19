use anyhow::Result;
use lightsql::{
    buffer::{BufferPoolManager, ClockSweepBufferPool},
    disk::{DiskManager, PageId},
    table::SimpleTable,
};
use md5::Md5;
use sha1::{Digest, Sha1};

// CREATE TABLE
// |id    |first_name|last_name|
// |------|----------|---------|
// |z     |Alice     |Smith    |
// |x     |Bob       |Johnson  |
// |y     |Charlie   |Williams |
// |w     |Dave      |Miller   |
// |v     |Eve       |Brown    |
// |...   |          |         |
// |BE i32|md5(id)   |sha1(id) |
fn main() -> Result<()> {
    let disk = DiskManager::open("simple.lsql")?;
    let pool = ClockSweepBufferPool::from(1_000_000);
    let mut bufmgr = BufferPoolManager::new(disk, pool);

    let mut table = SimpleTable {
        meta_page_id: PageId::new(0),
        key_elems_count: 1,
    };
    table.create(&mut bufmgr)?;
    dbg!(&table);
    table.insert(&mut bufmgr, &[b"z", b"Alice", b"Smith"])?;
    table.insert(&mut bufmgr, &[b"x", b"Bob", b"Johnson"])?;
    table.insert(&mut bufmgr, &[b"y", b"Charlie", b"Williams"])?;
    table.insert(&mut bufmgr, &[b"w", b"Dave", b"Miller"])?;
    table.insert(&mut bufmgr, &[b"v", b"Eve", b"Brown"])?;
    // for i in 1u32..=1_000_000u32 {
    //     let pkey = i.to_be_bytes();
    //     let md5 = Md5::digest(&pkey);
    //     let sha1 = Sha1::digest(&pkey);
    //     dbg!(i);
    //     table.insert(&mut bufmgr, &[&pkey[..], &md5[..], &sha1[..]])?;
    // }
    bufmgr.flush()?;
    Ok(())
}
