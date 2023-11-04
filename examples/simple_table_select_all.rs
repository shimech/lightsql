use anyhow::Result;
use lightsql::{
    btree::{BTree, SearchMode},
    buffer::{BufferPool, BufferPoolManager},
    disk::{DiskManager, PageId},
    tuple,
};

fn main() -> Result<()> {
    let disk = DiskManager::open("simple.lsql")?;
    let pool = BufferPool::new(10);
    let mut bufmgr = BufferPoolManager::new(disk, pool);

    let btree = BTree::new(PageId(0));
    let mut iter = btree.search(&mut bufmgr, SearchMode::Start)?;

    while let Some((key, value)) = iter.next(&mut bufmgr)? {
        let mut record = vec![];
        tuple::decode(&key, &mut record);
        tuple::decode(&value, &mut record);
        println!("{:?}", tuple::Pretty(&record));
    }
    Ok(())
}
