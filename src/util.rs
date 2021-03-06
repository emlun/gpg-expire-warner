pub struct Grouped<I>
where
    I: Iterator,
{
    n: usize,
    it: I,
}

impl<I> Iterator for Grouped<I>
where
    I: Iterator,
{
    type Item = Vec<I::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut nexts: Self::Item = Vec::new();

        loop {
            match self.it.next() {
                Some(next) => nexts.push(next),
                None => break,
            };
            if nexts.len() == self.n {
                break;
            }
        }
        if nexts.is_empty() {
            None
        } else {
            Some(nexts)
        }
    }
}

pub trait GroupedExt: Iterator {
    fn grouped(self, n: usize) -> Grouped<Self>
    where
        Self: Sized,
    {
        Grouped { n: n, it: self }
    }
}

impl<I: Iterator> GroupedExt for I {}
