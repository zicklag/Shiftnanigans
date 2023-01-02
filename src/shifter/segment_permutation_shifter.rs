use std::{rc::Rc, collections::VecDeque};
use bitvec::vec::BitVec;
use super::Shifter;

/// This struct is an unfixed line segment.
#[derive(Clone, Debug)]
pub struct Segment {
    length: usize
}

impl Segment {
    pub fn new(length: usize) -> Self {
        Segment {
            length: length
        }
    }
}

/// This struct is a fixed line segment.
#[derive(Clone, Debug)]
pub struct LocatedSegment {
    pub segment_index: usize,
    pub position: usize
}

impl LocatedSegment {
    pub fn new(segment_index: usize, position: usize) -> Self {
        LocatedSegment {
            segment_index: segment_index,
            position: position
        }
    }
}

pub struct SegmentPermutationShifter {
    segments: Rc<Vec<Segment>>,
    origin: (i32, i32),
    is_horizontal: bool,
    padding: usize,
    current_mask: BitVec,
    current_segment_index_per_shift_index: VecDeque<usize>,
    current_bounding_length_per_shift_index: VecDeque<usize>,
    current_minimum_bounding_length_per_shift_index: VecDeque<usize>,
    current_initial_position_offset_per_shift_index: VecDeque<usize>,
    current_position_offset_per_shift_index: VecDeque<Option<usize>>,
    current_remaining_maximum_bounding_length: usize,
    current_remaining_minimum_bounding_length: usize,
    is_shifted_outside: bool
}

impl SegmentPermutationShifter {
    pub fn new(segments: Rc<Vec<Segment>>, origin: (i32, i32), bounding_length: usize, is_horizontal: bool, padding: usize) -> Self {
        let mut current_mask: BitVec = BitVec::with_capacity(segments.len());
        current_mask.resize(segments.len(), false);

        let mut current_remaining_minimum_bounding_length: usize = 0;
        for (segment_index, segment) in segments.iter().enumerate() {
            if segment_index != 0 {
                current_remaining_minimum_bounding_length += padding;
            }
            current_remaining_minimum_bounding_length += segment.length;
        }

        SegmentPermutationShifter {
            segments: segments,
            origin: origin,
            is_horizontal: is_horizontal,
            padding: padding,
            current_mask: current_mask,
            current_segment_index_per_shift_index: VecDeque::new(),
            current_bounding_length_per_shift_index: VecDeque::new(),
            current_minimum_bounding_length_per_shift_index: VecDeque::new(),
            current_initial_position_offset_per_shift_index: VecDeque::new(),
            current_position_offset_per_shift_index: VecDeque::new(),
            current_remaining_maximum_bounding_length: bounding_length,
            current_remaining_minimum_bounding_length: current_remaining_minimum_bounding_length,
            is_shifted_outside: false
        }
    }
}

impl Shifter for SegmentPermutationShifter {
    type T = (i32, i32);

    fn try_forward(&mut self) -> bool {
        // calculate bounding length for the next "other segments"
        // calculate position of chosen segment in "other segments"

        if self.is_shifted_outside {
            return false;
        }
        else {
            if self.current_mask.count_zeros() == 0 {
                self.is_shifted_outside = true;
                return false;
            }
            else {
                for mask_index in 0..self.segments.len() {
                    if !self.current_mask[mask_index] {
                        // choose this segment index
                        self.current_mask.set(mask_index, true);
                        // specify that this shift index maps to this segment index
                        self.current_segment_index_per_shift_index.push_back(mask_index);
                        // store the starting and ending bounding length for this shift index
                        self.current_bounding_length_per_shift_index.push_back(self.current_remaining_maximum_bounding_length);
                        self.current_minimum_bounding_length_per_shift_index.push_back(self.current_remaining_minimum_bounding_length);
                        if self.current_mask.count_ones() == 1 {
                            self.current_initial_position_offset_per_shift_index.push_back(0);
                        }
                        else {
                            let previous_shift_index = self.current_initial_position_offset_per_shift_index.len() - 1;
                            let previous_segment_index = self.current_segment_index_per_shift_index[previous_shift_index];
                            let previous_segment_length = self.segments[previous_segment_index].length;
                            let previous_position_offset = self.current_position_offset_per_shift_index[previous_shift_index].unwrap();
                            let current_initial_position = previous_position_offset + previous_segment_length + self.padding;
                            self.current_initial_position_offset_per_shift_index.push_back(current_initial_position);
                        }

                        let segment_length = self.segments[mask_index].length;
                        self.current_remaining_maximum_bounding_length -= segment_length;
                        self.current_remaining_minimum_bounding_length -= segment_length;
                        if self.current_mask.count_zeros() != 0 {
                            self.current_remaining_maximum_bounding_length -= self.padding;
                            self.current_remaining_minimum_bounding_length -= self.padding;
                        }
                        return true;
                    }
                }
                panic!("Unexpected missing mask opening.");
            }
        }
    }
    fn try_backward(&mut self) -> bool {
        if self.current_mask.count_ones() == 0 {
            return false;
        }
        else {
            if self.is_shifted_outside {
                self.is_shifted_outside = false;
                return true;
            }
            else {
                let current_segment_index = self.current_segment_index_per_shift_index.pop_back().unwrap();
                self.current_mask.set(current_segment_index, false);
                self.current_remaining_maximum_bounding_length = self.current_bounding_length_per_shift_index.pop_back().unwrap();
                self.current_remaining_minimum_bounding_length = self.current_minimum_bounding_length_per_shift_index.pop_back().unwrap();
                return true;
            }
        }
    }
    fn try_increment(&mut self) -> bool {
        let current_shift_index = self.current_segment_index_per_shift_index.len() - 1;
        if self.current_position_offset_per_shift_index[current_shift_index].is_none() {
            self.current_position_offset_per_shift_index[current_shift_index] = Some(self.current_initial_position_offset_per_shift_index[current_shift_index]);
            return true;
        }
        else {
            let current_bounding_length = self.current_bounding_length_per_shift_index[current_shift_index];
            if current_bounding_length == self.current_minimum_bounding_length_per_shift_index[current_shift_index] {
                return false;
            }
            else {
                self.current_bounding_length_per_shift_index[current_shift_index] = current_bounding_length - 1;
                self.current_position_offset_per_shift_index[current_shift_index] = Some(self.current_position_offset_per_shift_index[current_shift_index].unwrap() + 1);
                return true;
            }
        }
    }
    fn get(&self) -> Option<Rc<(i32, i32)>> {
        let current_position_offset = self.current_position_offset_per_shift_index.back().unwrap().unwrap();
        if self.is_horizontal {
            return Some(Rc::new((self.origin.0 + current_position_offset as i32, self.origin.1)));
        }
        else {
            return Some(Rc::new((self.origin.0, self.origin.1 + current_position_offset as i32)));
        }
    }
}

#[cfg(test)]
mod segment_permutation_shifter_tests {
    use std::{time::{Duration, Instant}, cell::RefCell};

    use super::*;
    use rstest::rstest;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[rstest]
    fn initialized_no_segments() {
        init();
    
        let segments: Vec<Segment> = Vec::new();
        let _ = SegmentPermutationShifter::new(Rc::new(segments), (10, 100), 5, true, 1);
    }
}