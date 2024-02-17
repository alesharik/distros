use bootloader_api::info::MemoryRegion;

pub trait MergeMemoryRegions {
    fn merge_regions(self) -> MergeMemoryRegionsIter<Self> where Self: Iterator<Item = MemoryRegion>, Self: Sized;
}

impl<I> MergeMemoryRegions for I where I: Iterator<Item = MemoryRegion> {
    fn merge_regions(self) -> MergeMemoryRegionsIter<I> {
        MergeMemoryRegionsIter {
            iter: self,
            last: None
        }
    }
}

pub struct MergeMemoryRegionsIter<I: Iterator<Item = MemoryRegion>> {
    iter: I,
    last: Option<MemoryRegion>
}

impl<I: Iterator<Item = MemoryRegion>> Iterator for MergeMemoryRegionsIter<I> {
    type Item = MemoryRegion;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some(item) = self.iter.next() else {
                return self.last.take();
            };

            if let Some(lst) = self.last.take() {
                if lst.end == item.start && lst.kind == item.kind { // can merge regions
                    self.last = Some(MemoryRegion {
                        start: lst.start,
                        end: item.end,
                        kind: item.kind,
                    });
                } else {
                    self.last = Some(item);
                    return Some(lst)
                }
            } else {
                self.last = Some(item);
            }
        }
    }
}