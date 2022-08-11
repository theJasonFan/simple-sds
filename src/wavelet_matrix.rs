// FIXME documentation
//! Wavelet matrix from:
//!
//! > Claude, Navarro, Ordóñez: The wavelet matrix: An efficient wavelet tree for large alphabets.
//! > Information Systems, 2015.
//! > DOI: [10.1016/j.is.2014.06.002](https://doi.org/10.1016/j.is.2014.06.002)

use crate::bit_vector::BitVector;
use crate::ops::{Vector, Access, AccessIter, VectorIndex, BitVec, Rank, Select, SelectZero, PredSucc};
use crate::raw_vector::{RawVector, PushRaw};
use crate::serialize::Serialize;
use crate::bits;

use std::io::{Read, Write};
use std::io;

// FIXME tests

//-----------------------------------------------------------------------------

// FIXME document
// FIXME document construction
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WaveletMatrix {
    len: usize,
    data: Vec<BitVector>,
}

macro_rules! wavelet_matrix_from {
    ($t:ident) => {
        impl From<Vec<$t>> for WaveletMatrix {
            fn from(source: Vec<$t>) -> Self {
                let mut source = source;
                let max_value = source.iter().cloned().max().unwrap_or(0);
                let width = bits::bit_len(max_value as u64);
        
                let mut data: Vec<BitVector> = Vec::new();
                for level in 0..width {
                    let bit_value: $t = 1 << (width - 1 - level);
                    let mut zeros: Vec<$t> = Vec::new();
                    let mut ones: Vec<$t> = Vec::new();
                    let mut raw_data = RawVector::with_capacity(source.len());
        
                    // Determine if the current bit is set in each value.
                    for value in source.iter() {
                        if value & bit_value != 0 {
                            ones.push(*value);
                            raw_data.push_bit(true);
                        } else {
                            zeros.push(*value);
                            raw_data.push_bit(false);
                        }
                    }
        
                    // Sort the values stably by the current bit.
                    source.clear();
                    source.extend(zeros);
                    source.extend(ones);
        
                    // Create the bitvector for the current level.
                    let mut bv = BitVector::from(raw_data);
                    bv.enable_rank();
                    bv.enable_select();
                    bv.enable_select_zero();
                    bv.enable_pred_succ();
                    data.push(bv);
                }
        
                WaveletMatrix {
                    len: source.len(),
                    data,
                }
            }
        }
    }
}

wavelet_matrix_from!(u8);
wavelet_matrix_from!(u16);
wavelet_matrix_from!(u32);
wavelet_matrix_from!(u64);
wavelet_matrix_from!(usize);

//-----------------------------------------------------------------------------

impl Vector for WaveletMatrix {
    type Item = u64;

    #[inline]
    fn len(&self) -> usize {
        self.len
    }

    #[inline]
    fn width(&self) -> usize {
        self.data.len()
    }

    #[inline]
    fn max_len(&self) -> usize {
        usize::MAX
    }
}

impl<'a> Access<'a> for WaveletMatrix {
    type Iter = AccessIter<'a, Self>;

    fn get(&self, index: usize) -> <Self as Vector>::Item {
        let mut index = index;
        let mut result = 0;
        for level in 0..self.width() {
            if self.data[level].get(index) {
                index = self.data[level].count_zeros() + self.data[level].rank(index);
                result += 1 << (self.width() - 1 - level);
            } else {
                index = self.data[level].rank_zero(index);
            }
        }
        result
    }

    fn iter(&'a self) -> Self::Iter {
        Self::Iter::new(self)
    }
}

// FIXME VectorIndex

// FIXME document serialization format
impl Serialize for WaveletMatrix {
    fn serialize_header<T: Write>(&self, writer: &mut T) -> io::Result<()> {
        self.len.serialize(writer)
    }

    fn serialize_body<T: Write>(&self, writer: &mut T) -> io::Result<()> {
        let width = self.data.len();
        width.serialize(writer)?;
        for bv in self.data.iter() {
            bv.serialize(writer)?;
        }
        Ok(())
    }

    fn load<T: Read>(reader: &mut T) -> io::Result<Self> {
        let len = usize::load(reader)?;
        let width = usize::load(reader)?;
        let mut data: Vec<BitVector> = Vec::new();
        for _ in 0..width {
            let mut bv = BitVector::load(reader)?;
            bv.enable_rank();
            bv.enable_select();
            bv.enable_select_zero();
            bv.enable_pred_succ();
            data.push(bv);
        }
        Ok(WaveletMatrix {
            len,
            data,
        })
    }

    fn size_in_elements(&self) -> usize {
        let mut result = self.len.size_in_elements();
        result += 1; // Width.
        for bv in self.data.iter() {
            result += bv.size_in_elements();
        }
        result
    }
}

//-----------------------------------------------------------------------------

// FIXME an iterator based on iterators over each level

//-----------------------------------------------------------------------------

// FIXME ValueIter

//-----------------------------------------------------------------------------

// FIXME IntoIter

//-----------------------------------------------------------------------------
