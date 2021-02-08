/*
*   Copyright (c) 2021 Neil F Jones
*   All rights reserved.

*   Permission is hereby granted, free of charge, to any person obtaining a copy
*   of this software and associated documentation files (the "Software"), to deal
*   in the Software without restriction, including without limitation the rights
*   to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
*   copies of the Software, and to permit persons to whom the Software is
*   furnished to do so, subject to the following conditions:

*   The above copyright notice and this permission notice shall be included in all
*   copies or substantial portions of the Software.

*   THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
*   IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
*   FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
*   AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
*   LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
*   OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
*   SOFTWARE.
*/
use crate::errorln;
use std::io::Read;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct ScanBuffer<T> {
    index: usize,
    size: usize,
    scan_size: usize,
    default_value: T,
    buffer: Vec<T>,
}

impl<T> ScanBuffer<T>
where
    T: Copy + From<u8>,
{
    pub fn new(size: usize, scan_size: usize, default_value: T) -> ScanBuffer<T> {
        let buffer = vec![default_value; size];
        let index = 0;
        return ScanBuffer {
            index,
            size,
            scan_size,
            default_value,
            buffer,
        };
    }

    pub fn shift<F>(&mut self, stream: &mut F) -> usize
    where
        F: Read,
    {
        let mut buffer: Vec<u8> = vec![b'\0'; self.scan_size];
        let mut bytes_read: usize = 0;

        stream
            .take(self.scan_size as u64)
            .read(&mut buffer)
            .and_then(|count: usize| -> Result<(), std::io::Error> {
                bytes_read = count;
                if bytes_read > 0 {
                    let mut data: Vec<T> = buffer.iter().map(|&b| -> T { T::from(b) }).collect();
                    self.buffer.append(&mut data);
                    self.buffer.rotate_left(bytes_read);
                    self.buffer.truncate(self.size);
                }
                return Ok(());
            })
            .or_else(|error| {
                errorln!("{}", error);
                return Err(error);
            })
            .ok();
        return bytes_read;
    }

    pub fn process<R, F: Fn(&Vec<T>) -> R>(&self, func: F) -> R {
        return func(&self.buffer);
    }
}
