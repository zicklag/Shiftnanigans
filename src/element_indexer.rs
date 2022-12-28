use std::{rc::Rc, collections::BTreeMap, cell::RefCell};

use crate::{segment_container::{SegmentPermutationIncrementer}, index_incrementer::IndexIncrementer};

pub trait ElementIndexer {
    type T;
    fn try_get_next_elements(&mut self) -> Option<Vec<Rc<Self::T>>>;
    fn reset(&mut self);
}

pub struct SegmentPermutationIncrementerElementIndexer<'a> {
    segment_permutation_incrementer: SegmentPermutationIncrementer<'a>,
    origin_location: (i32, i32),
    is_horizontal: bool,
    calculated_location_per_position: BTreeMap<usize, Rc<(i32, i32)>>
}

impl<'a> ElementIndexer for SegmentPermutationIncrementerElementIndexer<'a> {
    type T = (i32, i32);

    fn try_get_next_elements(&mut self) -> Option<Vec<Rc<Self::T>>> {
        let segment_location_permutations_option = self.segment_permutation_incrementer.try_get_next_segment_location_permutations();
        if let Some(segment_location_permutations) = segment_location_permutations_option {
            let mut elements: Vec<Rc<(i32, i32)>> = Vec::new();
            for segment_location in segment_location_permutations.iter() {
                let is_calculated_location_cached = self.calculated_location_per_position.contains_key(&segment_location.position);
                if !is_calculated_location_cached {
                    let element: (i32, i32);
                    if self.is_horizontal {
                        element = (self.origin_location.0 + segment_location.position as i32, self.origin_location.1);
                    }
                    else {
                        element = (self.origin_location.0, self.origin_location.1 + segment_location.position as i32);
                    }
                    self.calculated_location_per_position.insert(segment_location.position, Rc::new(element));
                }
                elements.push(Rc::new((0, 0)));
            }
            for segment_location in segment_location_permutations.into_iter() {
                elements[segment_location.segment_index] = self.calculated_location_per_position.get(&segment_location.position).unwrap().clone();
            }
            return Some(elements);
        }
        return None;
    }
    fn reset(&mut self) {
        self.segment_permutation_incrementer.reset();
    }
}

impl<'a> SegmentPermutationIncrementerElementIndexer<'a> {
    pub fn new(segment_permutation_incrementer: SegmentPermutationIncrementer<'a>, origin_location: (i32, i32), is_horizontal: bool) -> Self {
        SegmentPermutationIncrementerElementIndexer {
            segment_permutation_incrementer: segment_permutation_incrementer,
            origin_location: origin_location,
            is_horizontal: is_horizontal,
            calculated_location_per_position: BTreeMap::new()
        }
    }
}

pub struct IndexIncrementerElementIndexer<TElement> {
    index_incrementer: IndexIncrementer,
    locations_per_element: Vec<Vec<Rc<TElement>>>,
    is_last_increment_successful: bool
}

impl<TElement> ElementIndexer for IndexIncrementerElementIndexer<TElement> {
    type T = TElement;

    fn try_get_next_elements(&mut self) -> Option<Vec<Rc<Self::T>>> {
        if !self.is_last_increment_successful {
            return None;
        }
        let location_index_per_element_index = self.index_incrementer.get();
        self.is_last_increment_successful = self.index_incrementer.try_increment();

        let mut elements: Vec<Rc<TElement>> = Vec::new();
        for (element_index, location_index) in location_index_per_element_index.iter().enumerate() {
            let locations = &self.locations_per_element[element_index];
            let element = &locations[location_index.unwrap()];
            elements.push(element.clone());
        }
        return Some(elements);
    }
    fn reset(&mut self) {
        self.index_incrementer.reset();
    }
}

impl<TElement> IndexIncrementerElementIndexer<TElement> {
    pub fn new(locations_per_element: Vec<Vec<TElement>>) -> Self {

        let mut rc_locations_per_element: Vec<Vec<Rc<TElement>>> = Vec::new();
        for locations in locations_per_element.into_iter() {
            let mut rc_locations: Vec<Rc<TElement>> = Vec::new();
            for element in locations.into_iter() {
                rc_locations.push(Rc::new(element));
            }
            rc_locations_per_element.push(rc_locations);
        }

        let index_incrementer = IndexIncrementer::from_vector_of_vector(&rc_locations_per_element);
        IndexIncrementerElementIndexer {
            index_incrementer: index_incrementer,
            locations_per_element: rc_locations_per_element,
            is_last_increment_successful: true
        }
    }
}

pub struct ElementIndexerIncrementer<T> {
    element_indexers: Vec<Box<dyn ElementIndexer<T = T>>>,
    previous_elements: Vec<Option<Rc<Vec<Rc<T>>>>>
}

impl<T: Clone + std::fmt::Debug> ElementIndexerIncrementer<T> {
    pub fn new(element_indexers: Vec<Box<dyn ElementIndexer<T = T>>>) -> Self {
        let mut previous_elements: Vec<Option<Rc<Vec<Rc<T>>>>> = Vec::new();
        for _ in element_indexers.iter() {
            previous_elements.push(None);
        }
        ElementIndexerIncrementer {
            element_indexers: element_indexers,
            previous_elements: previous_elements
        }
    }
    pub fn try_get_next_elements(&mut self) -> Option<Vec<Rc<T>>>{
        let mut elements: Vec<Rc<T>> = Vec::new();
        let mut is_previous_element_indexer_incremented: bool = false;
        let mut is_last_element_indexer_cycled: bool = self.element_indexers.is_empty();
        let mut element_indexer_index: usize = 0;
        let element_indexers_length: usize = self.element_indexers.len();
        while element_indexer_index < element_indexers_length {
            if self.previous_elements[element_indexer_index].is_none() {
                let elements = self.element_indexers[element_indexer_index].try_get_next_elements().unwrap();
                self.previous_elements[element_indexer_index] = Some(Rc::new(elements));
            }
            else {
                if !is_previous_element_indexer_incremented {
                    let elements_option = self.element_indexers[element_indexer_index].try_get_next_elements();
                    if elements_option.is_none() {
                        self.element_indexers[element_indexer_index].reset();
                        let elements = self.element_indexers[element_indexer_index].try_get_next_elements().unwrap();
                        self.previous_elements[element_indexer_index] = Some(Rc::new(elements));
                        if element_indexer_index + 1 == element_indexers_length {
                            is_last_element_indexer_cycled = true;
                        }
                    }
                    else {
                        self.previous_elements[element_indexer_index] = Some(Rc::new(elements_option.unwrap()));
                        is_previous_element_indexer_incremented = true;
                    }
                }
            }
            for element in self.previous_elements[element_indexer_index].as_ref().unwrap().iter() {
                elements.push(element.clone());
            }
            element_indexer_index += 1;
        }
        if is_last_element_indexer_cycled {
            None
        }
        else {
            println!("elements: {:?}", elements);
            Some(elements)
        }
    }
}

#[cfg(test)]
mod element_indexer_tests {
    use crate::segment_container::Segment;

    use super::*;
    use rstest::rstest;

    fn init() {
        std::env::set_var("RUST_LOG", "trace");
        //pretty_env_logger::try_init();
    }

    #[rstest]
    #[case(1)]
    #[case(2)]
    #[case(3)]
    #[case(4)]
    #[case(5)]
    fn initialize_segment_permutation_incrementer_element_indexer_with_initialized_element_indexes(#[case] segments_total: usize) {
        init();

        let mut segments: Vec<Segment> = Vec::new();
        for segment_index in 0..segments_total {
            segments.push(Segment::new(segment_index + 1));
        }

        let bounding_length: usize = (segments_total * (segments_total + 1)) / 2 as usize + (segments_total - 1);
        let padding: usize = 1;

        let segment_permutation_incrementer: SegmentPermutationIncrementer = SegmentPermutationIncrementer::new(&segments, bounding_length, padding);
        let origin_location: (i32, i32) = (1, 2);
        let is_horizontal: bool = true;

        let _: SegmentPermutationIncrementerElementIndexer = SegmentPermutationIncrementerElementIndexer::new(segment_permutation_incrementer, origin_location, is_horizontal);
    }

    #[rstest]
    #[case(3)]
    fn get_elements_from_specific_segment_permutation_incrementer_element_indexer(#[case] segments_total: usize) {
        init();

        let mut segments: Vec<Segment> = Vec::new();
        for segment_index in 0..segments_total {
            segments.push(Segment::new(segment_index + 1));
        }

        let bounding_length: usize = (segments_total * (segments_total + 1)) / 2 as usize + (segments_total - 1);
        let padding: usize = 1;

        let segment_permutation_incrementer: SegmentPermutationIncrementer = SegmentPermutationIncrementer::new(&segments, bounding_length, padding);
        let origin_location: (i32, i32) = (10, 100);
        let is_horizontal: bool = true;

        let mut element_indexer: SegmentPermutationIncrementerElementIndexer = SegmentPermutationIncrementerElementIndexer::new(segment_permutation_incrementer, origin_location, is_horizontal);

        let elements_option = element_indexer.try_get_next_elements();
        println!("elements_option: {:?}", elements_option);
        assert!(elements_option.is_some());

        let elements = elements_option.unwrap();
        assert_eq!(3, elements.len());
        assert_eq!(&(10, 100), elements[0].as_ref());
        assert_eq!(&(12, 100), elements[1].as_ref());
        assert_eq!(&(15, 100), elements[2].as_ref());

        let elements_option = element_indexer.try_get_next_elements();
        println!("elements_option: {:?}", elements_option);
        assert!(elements_option.is_some());

        let elements = elements_option.unwrap();
        assert_eq!(3, elements.len());
        assert_eq!(&(10, 100), elements[0].as_ref());
        assert_eq!(&(16, 100), elements[1].as_ref());
        assert_eq!(&(12, 100), elements[2].as_ref());

        let elements_option = element_indexer.try_get_next_elements();
        println!("elements_option: {:?}", elements_option);
        assert!(elements_option.is_some());

        let elements = elements_option.unwrap();
        assert_eq!(3, elements.len());
        assert_eq!(&(13, 100), elements[0].as_ref());
        assert_eq!(&(10, 100), elements[1].as_ref());
        assert_eq!(&(15, 100), elements[2].as_ref());

        let elements_option = element_indexer.try_get_next_elements();
        println!("elements_option: {:?}", elements_option);
        assert!(elements_option.is_some());

        let elements = elements_option.unwrap();
        assert_eq!(3, elements.len());
        assert_eq!(&(17, 100), elements[0].as_ref());
        assert_eq!(&(10, 100), elements[1].as_ref());
        assert_eq!(&(13, 100), elements[2].as_ref());

        let elements_option = element_indexer.try_get_next_elements();
        println!("elements_option: {:?}", elements_option);
        assert!(elements_option.is_some());

        let elements = elements_option.unwrap();
        assert_eq!(3, elements.len());
        assert_eq!(&(14, 100), elements[0].as_ref());
        assert_eq!(&(16, 100), elements[1].as_ref());
        assert_eq!(&(10, 100), elements[2].as_ref());

        let elements_option = element_indexer.try_get_next_elements();
        println!("elements_option: {:?}", elements_option);
        assert!(elements_option.is_some());

        let elements = elements_option.unwrap();
        assert_eq!(3, elements.len());
        assert_eq!(&(17, 100), elements[0].as_ref());
        assert_eq!(&(14, 100), elements[1].as_ref());
        assert_eq!(&(10, 100), elements[2].as_ref());

        let elements_option = element_indexer.try_get_next_elements();
        println!("elements_option: {:?}", elements_option);
        assert!(elements_option.is_none());
    }

    #[rstest]
    #[case(1)]
    #[case(2)]
    #[case(3)]
    #[case(4)]
    #[case(5)]
    #[case(10)]
    fn get_elements_from_segment_permutation_incrementer_element_indexer(#[case] segments_total: usize) {
        init();

        let mut segments: Vec<Segment> = Vec::new();
        for segment_index in 0..segments_total {
            segments.push(Segment::new(segment_index + 1));
        }

        let bounding_length: usize = (segments_total * (segments_total + 1)) / 2 as usize + (segments_total - 1);
        let padding: usize = 1;

        let segment_permutation_incrementer: SegmentPermutationIncrementer = SegmentPermutationIncrementer::new(&segments, bounding_length, padding);
        let origin_location: (i32, i32) = (1, 2);
        let is_horizontal: bool = true;

        let mut element_indexer: SegmentPermutationIncrementerElementIndexer = SegmentPermutationIncrementerElementIndexer::new(segment_permutation_incrementer, origin_location, is_horizontal);

        let mut is_successful = true;
        let mut iterations_total = 0;
        while is_successful {
            is_successful = element_indexer.try_get_next_elements().is_some();
            if is_successful {
                iterations_total += 1;
            }
        }

        let mut expected_iterations_total = 1;
        for segment_index in 0..segments_total {
            expected_iterations_total *= segment_index + 1;
        }
        assert_eq!(expected_iterations_total, iterations_total);
    }
    #[rstest]
    fn get_element_indexer_incrementer_zero_element_indexers() {
        let element_indexers: Vec<Box<dyn ElementIndexer<T = String>>> = Vec::new();
        let mut element_indexer_incrementer = ElementIndexerIncrementer::new(element_indexers);
        let elements_option = element_indexer_incrementer.try_get_next_elements();
        assert!(elements_option.is_none());
    }
    #[rstest]
    fn get_element_indexer_incrementer_one_element_indexer_index_incrementer() {
        let mut element_indexers: Vec<Box<dyn ElementIndexer<T = String>>> = Vec::new();
        element_indexers.push(Box::new(IndexIncrementerElementIndexer::new(
            vec![vec![String::from("1/3"), String::from("2/3"), String::from("3/3")], vec![String::from("1/2"), String::from("2/2")]]
        )));
        let mut element_indexer_incrementer = ElementIndexerIncrementer::new(element_indexers);
        let elements_option = element_indexer_incrementer.try_get_next_elements();
        assert!(elements_option.is_some());
        let elements = elements_option.unwrap();
        assert_eq!(&String::from("1/3"), elements[0].as_ref());
        //assert!(elements_option.is_none());
    }
}