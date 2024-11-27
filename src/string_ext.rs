pub trait StringExt {
    fn substring(&self, start: usize, end: usize) -> String;
    fn char_at(&self, index: usize) -> char;
}

impl StringExt for String {
    fn substring(&self, start: usize, end: usize) -> String {
        self.chars().skip(start).take(end - start).collect()
    }

    fn char_at(&self, index: usize) -> char {
        self.chars().nth(index).unwrap_or_default()
    }
}
