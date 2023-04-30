// rlox: Lox interpreter/compiler in Rust.
// Copyright (C) 2023  James Smyle <j@mes.sh>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use unicode_segmentation::UnicodeSegmentation;

pub trait PeekableIterator: Iterator {
    fn peek(&self) -> Option<Self::Item>;
}

pub struct Input<'a>(Vec<&'a str>);

impl<'a> Input<'a> {
    pub fn new(source: &'a str) -> Self {
        Self(UnicodeSegmentation::graphemes(source, true).collect())
    }
}

// Custom iterator type
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

pub struct InputIter<'a> {
    input: Input<'a>,
    index: usize,
    line_num: usize,
    line_pos: usize,
}

impl<'a> InputIter<'a> {
    pub fn line(&self) -> usize {
        self.line_num
    }
    pub fn location(&self) -> usize {
        self.line_pos
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
