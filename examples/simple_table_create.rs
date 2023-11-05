use anyhow::Result;
use lightsql::{
    buffer::{BufferPoolManager, ClockSweepBufferPool},
    disk::{DiskManager, PageId},
    table::SimpleTable,
};

fn main() -> Result<()> {
    let disk = DiskManager::open("simple.lsql")?;
    let pool = ClockSweepBufferPool::from(10);
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

    bufmgr.flush()?;
    Ok(())
}
