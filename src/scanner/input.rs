use unicode_segmentation::UnicodeSegmentation;

pub trait PeekableIterator: Iterator {
    fn peek(&self) -> Option<Self::Item>;
}

pub struct Input<'a>(Vec<&'a str>);

impl<'a> Input<'a> {
    pub fn new(source: &'a str) -> Self {
        let graphemes = UnicodeSegmentation::graphemes(source, true).collect();
        Self(graphemes)
    }
}

pub struct InputIter<'a> {
    input: Input<'a>,
    index: usize,
    line_num: usize,
    line_pos: usize,
}

impl<'a> IntoIterator for Input<'a> {
    type Item = &'a str;
    type IntoIter = InputIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            line_num: 1,
            line_pos: 1,
            index: 0,
            input: self,
        }
    }
}

impl<'a> Iterator for InputIter<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.input.0.len() {
            None
        } else {
            let next_grapheme = self.input.0[self.index];

            match next_grapheme {
                "\n" => {
                    self.line_num += 1;
                    self.line_pos = 1;
                }
                _ => {
                    self.line_pos += 1;
                }
            }

            self.index += 1;
            Some(next_grapheme)
        }
    }
}

impl<'a> PeekableIterator for InputIter<'a> {
    fn peek(&self) -> Option<Self::Item> {
        if self.index >= self.input.0.len() {
            None
        } else {
            Some(self.input.0[self.index])
        }
    }
}

impl<'a> InputIter<'a> {
    pub fn line(&self) -> usize {
        self.line_num
    }
    pub fn location(&self) -> usize {
        self.line_pos
    }
}
