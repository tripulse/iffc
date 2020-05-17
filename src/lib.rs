//! IFF is a binary-interchange format developed by Electronic Arts
//! for tagging binary data with a meaning. This file is made of out
//! of segments referred to as so called "chunks". This format is used
//! for mainly storing multimedia, eg. audio, video, midi, images.
//! 
//! This crate provides data-structures and wrappers to manipulate this
//! format quite easily by reading and decoding or writing and encoding
//! from or into file-streams.
//! 
//! # Examples
//! To decode all the chunks avialable from the given reader:
//! ```
//! use iffc::Decoder;
//! 
//! fn main() {
//!     let inp = std::io::Cursor::new(b"RIFF\x04\x00\x00\x00WAVE");
//!     let parser = Decoder(Box::new(inp));
//! 
//!     for chk in inp
//!     { println!("{:?}: {}", chk.0, chk.1.len()); }
//! }
//! ```
//! 
//! To encode chunks into a given writer:
//! ```
//! use iffc::{Encoder, Chunk};
//! 
//! fn main() {
//!     let out = std::io::Cursor::new(Vec::new());
//!     let deparser = Encoder(Box::new(out));
//! 
//!     deparser << Chunk(*b"RIFF", b"WAVE".to_vec());
//! }
//! ```
use std::io::{Read, Write, IoSlice, IoSliceMut};
use std::ops::{Shl};

/// An IFF chunk represents a single segment of a complete IFF
/// file. Note: Even though this structure is capable of stroing
/// data upto `usize` but IFF limits that to `u32` only.
/// 
/// `0` — four-byte identity of chunk.
/// `1` — byte-data encapsulated inside it.
#[derive(Debug, Eq, PartialEq)]
pub struct Chunk(pub [u8; 4], pub Vec<u8>);

/// A structure which wraps a reader and parses IFF chunks and
/// behaves like an iterator which yields `IFFChunk` until
/// an entire-chunk can't be constructed.
pub struct Decoder(Box<dyn Read>);

/// A structure which wraps a writer and writes IFF chunks to it,
/// by using `<<` (shift-left) with an RHS of type `IFFChunk`, also
/// that operand can be chained.
pub struct Encoder(Box<dyn Write>);

impl Iterator for Decoder {
    type Item = Chunk;

    fn next(&mut self) -> Option<Self::Item> {
        let mut id   = [0u8; 4];
        let mut size = [0u8; 4];

        if let Err(_) = self.0.read_vectored(&mut [
            IoSliceMut::new(&mut id),
            IoSliceMut::new(&mut size)
        ]) { return None };

        let size = u32::from_le_bytes(size) as usize;
        let mut data = vec![0u8; size];
        
        match self.0.read(&mut data) {
            Ok(s) => if size != s || s == 0
                     { return None },
            Err(_) => { return None }
        };

        Some(Chunk(id, data))
    }
}

impl Shl<Chunk> for Encoder {
    type Output = Option<Self>;

    fn shl(self, chunk: Chunk) -> Option<Self> {
        let mut sel = self;
        
        match sel.0.write_vectored(&[
            IoSlice::new(&chunk.0),
            IoSlice::new(&(chunk.1.len() as u32)
                            .to_le_bytes()[..]),
            IoSlice::new(chunk.1.as_slice())
        ]) { Ok(_) => Some(Self(sel.0)),
             Err(_) => None }
    }
}