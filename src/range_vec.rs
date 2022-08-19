use std::ops::Range;

#[derive(Debug,Clone)]
pub struct RangeVec {
    ranges: Vec<Range<usize>>,
}

impl RangeVec
{
    pub fn new() -> Self {
        RangeVec { ranges: Vec::new() }
    }

    pub fn push(&mut self, data: usize) {
        // to find the a position to insert our range
        let result = self.ranges
            .iter()
            .enumerate()
            .filter(|(_, range)| data >= range.start)
            .last()
            .map(|(i,_)| i);
        if let Some(index) = result {
            let current = unsafe { self.ranges.get_unchecked_mut(index) };
            if current.contains(&data) {
                return;
            }
            if current.end == data {
                current.end += 1;
                return;
            }
            std::mem::drop(current);
            self.ranges.insert(index - 1,data..(data + 1));
        } else {
            self.ranges.insert(0,data..(data+1));
        }
    }

    /// Remove a data in RangeVec
    /// # Details
    /// * [1..5] remove 3 will be [1..3][4..5]
    pub fn remove(&mut self,data: usize) {
        let result = self.ranges.iter()
            .enumerate()
            .find(|(_,range)|range.contains(&data))
            .map(|(i,_)|i);
        if let Some(index) = result {
            // # Safety
            // * index is valid because it is from enumerate()
            let current = unsafe {
                self.ranges.get_unchecked_mut(index)
            };

            let current_end = current.end;
            current.end = data;

            if data != current_end - 1 {
                let next = (data+1)..current_end;
                self.ranges.push(next);
            }
            
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RangeVec;

    #[test]
    fn basic_test() {
        let mut v = RangeVec::new();
        v.push(5);
        assert_eq!(&v.ranges,&[5..6]);
        v.push(6);
        assert_eq!(&v.ranges,&[5..7]);
        v.push(7);
        assert_eq!(&v.ranges,&[5..8]);
        v.push(3);
        assert_eq!(&v.ranges,&[3..4,5..8]);
        v.push(2);
        assert_eq!(&v.ranges,&[2..4,5..8]);
    }
}