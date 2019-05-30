pub struct Grouped<I, T>
where
    I: Iterator<Item = T>,
{
    n: usize,
    it: I,
}

impl<I, T> Iterator for Grouped<I, T>
where
    I: Iterator<Item = T>,
{
    type Item = Vec<T>;
    fn next(&mut self) -> Option<Vec<T>> {
        let mut nexts: Vec<T> = Vec::new();

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

pub fn grouped<T, I: Iterator<Item = T>>(n: usize, it: I) -> Grouped<I, T> {
    Grouped { n: n, it: it }
}
