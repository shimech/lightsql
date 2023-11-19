use anyhow::Result;
use lightsql::{
    buffer::{BufferPoolManager, ClockSweepBufferPool},
    disk::{DiskManager, PageId},
    query::{Filter, PlanNode, SeqScan, TupleSearchMode},
    tuple,
};

fn main() -> Result<()> {
    let disk = DiskManager::open("simple.lsql")?;
    let pool = ClockSweepBufferPool::from(10);
    let mut bufmgr = BufferPoolManager::new(disk, pool);

    let plan = Filter {
        cond: &|record| record[0].as_slice() == b"y",
        inner_plan: &SeqScan {
            table_meta_page_id: PageId::new(0),
            search_mode: TupleSearchMode::Start,
            while_cond: &|_| true,
        },
    };
    let mut exec = plan.start(&mut bufmgr)?;

    while let Some(record) = exec.next(&mut bufmgr)? {
        println!("{:?}", tuple::Pretty(&record));
    }
    Ok(())
}
