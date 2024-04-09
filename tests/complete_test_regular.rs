use basic_trie::Trie;
use growable_bloom_filter::GrowableBloom;
use peak_alloc::PeakAlloc;
use randomizer::Randomizer;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Instant;
use unicode_segmentation::UnicodeSegmentation;

#[global_allocator]
static PEAK_ALLOC: PeakAlloc = PeakAlloc;

pub struct BigData {
    pub number_of_words: usize,
    pub first_letter_histogram: HashMap<String, usize>,
    pub big_data: Vec<String>,
}

pub fn generate_random_lines(x: usize, y: usize) -> BigData {
    let mut bloom_filter = GrowableBloom::new(0.01, x);
    let mut result = Vec::new();
    let mut first_letter_histogram = HashMap::<String, usize>::new();
    let mut number_of_words = 0;

    for _ in 0..x {
        let random_string = Randomizer::ALPHABETICAL_LOWER(y).string().unwrap();
        if !bloom_filter.contains(&random_string) {
            bloom_filter.insert(&random_string);

            let first_letter = random_string[0..1].to_string();
            *first_letter_histogram.entry(first_letter).or_insert(0) += 1;

            result.push(random_string);
            number_of_words += 1;
        }
    }

    BigData {
        number_of_words,
        first_letter_histogram,
        big_data: result,
    }
}

#[test]
fn overall_regular() {
    let number_of_words = 500_000;
    let word_length = 15;

    let big_data = generate_random_lines(number_of_words, word_length);

    println!(
        "Memory usage after loading words: {:.1}mb\n",
        PEAK_ALLOC.current_usage_as_mb()
    );

    let mut trie = Trie::new();

    for word in big_data.big_data {
        trie.insert(&word);
    }

    println!(
        "Memory usage after trie generation: {:.1}mb",
        PEAK_ALLOC.current_usage_as_mb()
    );

    let now = Instant::now();

    assert_eq!(trie.len(), big_data.number_of_words);
    assert_eq!(word_length, trie.get_longest()[0].graphemes(true).count());
    assert_eq!(word_length, trie.get_shortest()[0].graphemes(true).count());

    for (first_letter, count) in big_data.first_letter_histogram.iter() {
        assert_eq!(*count, trie.len_prefix(first_letter));
    }

    let elapsed = now.elapsed();
    println!("Operations time: {:.2?}", elapsed);
}

#[test]
fn clearing_regular() {
    let number_of_words = 500_000;
    let word_length = 15;

    let big_data = generate_random_lines(number_of_words, word_length);

    println!(
        "Memory usage after loading words: {:.1}mb\n",
        PEAK_ALLOC.current_usage_as_mb()
    );

    let mut trie = Trie::new();

    for word in big_data.big_data {
        trie.insert(&word);
    }

    println!(
        "Memory usage after trie generation: {:.1}mb\n",
        PEAK_ALLOC.current_usage_as_mb()
    );

    let now = Instant::now();

    trie.clear();
    assert!(trie.is_empty());

    let elapsed = now.elapsed();
    println!("Operations time: {:.2?}\n", elapsed);

    println!(
        "Memory usage after trie cleanup: {:.1}mb\n",
        PEAK_ALLOC.current_usage_as_mb()
    );
}

#[test]
fn add_op_regular_1() {
    let number_of_words = 100_000;
    let word_length = 15;

    let big_data = generate_random_lines(number_of_words, word_length);

    let mut trie_0 = Trie::new();
    let mut trie_1 = Trie::new();
    let mut trie_2 = Trie::new();

    println!(
        "Memory usage after loading words: {:.1}mb\n",
        PEAK_ALLOC.current_usage_as_mb()
    );

    for line in big_data.big_data.iter() {
        trie_0.insert(line);
    }

    for line in big_data
        .big_data
        .iter()
        .rev()
        .skip(big_data.number_of_words / 2)
    {
        trie_1.insert(line);
    }

    for line in big_data.big_data.iter().skip(big_data.number_of_words / 2) {
        trie_2.insert(line);
    }

    println!(
        "Memory usage after trie generation: {:.1}mb",
        PEAK_ALLOC.current_usage_as_mb()
    );

    let now = Instant::now();

    trie_1 += trie_2;

    let elapsed = now.elapsed();
    println!("Operations time: {:.2?}", elapsed);

    let t1_words = trie_1.get_all();
    let correct_words = trie_0.get_all();

    let item_set: HashSet<_> = t1_words.iter().collect();
    let only_in_correct: Vec<_> = correct_words
        .into_iter()
        .filter(|item| !item_set.contains(item))
        .collect();

    assert_eq!(only_in_correct, Vec::<String>::new());
    assert_eq!(only_in_correct.len(), 0);
    assert!(trie_0 == trie_1);

    println!(
        "Memory usage after trie addition: {:.1}mb\n",
        PEAK_ALLOC.current_usage_as_mb()
    );
}

#[test]
fn add_op_regular_2() {
    let number_of_words = 100_000;
    let word_length = 15;

    let big_data = generate_random_lines(number_of_words, word_length);

    let mut trie_0 = Trie::new();
    let mut trie_1 = Trie::new();
    let mut trie_2 = Trie::new();

    println!(
        "Memory usage after loading words: {:.1}mb\n",
        PEAK_ALLOC.current_usage_as_mb()
    );

    for line in big_data.big_data.iter() {
        trie_0.insert(&line);
    }

    for line in big_data.big_data.iter().rev().skip(20000) {
        trie_1.insert(&line);
    }

    for line in big_data.big_data.iter().skip(20000) {
        trie_2.insert(&line);
    }

    println!(
        "Memory usage after trie generation: {:.1}mb",
        PEAK_ALLOC.current_usage_as_mb()
    );

    let now = Instant::now();

    let trie_3 = trie_1 + trie_2;
    let elapsed = now.elapsed();
    println!("Operations time: {:.2?}", elapsed);

    assert!(trie_0 == trie_3);

    let t3_words = trie_3.get_all();
    let correct_words = trie_0.get_all();

    let item_set: HashSet<_> = t3_words.iter().collect();
    let only_in_correct: Vec<_> = correct_words
        .into_iter()
        .filter(|item| !item_set.contains(item))
        .collect();

    assert_eq!(only_in_correct.len(), 0);

    println!(
        "Memory usage after trie addition: {:.1}mb\n",
        PEAK_ALLOC.current_usage_as_mb()
    );
}

#[test]
fn equals_regular() {
    let number_of_words = 100_000;
    let word_length = 15;

    let big_data = generate_random_lines(number_of_words, word_length);

    let mut trie_1 = Trie::new();
    let mut trie_2 = Trie::new();

    println!(
        "Memory usage after loading words: {:.1}mb\n",
        PEAK_ALLOC.current_usage_as_mb()
    );

    for line in big_data.big_data.iter() {
        trie_1.insert(&line);
    }

    for line in big_data.big_data.iter() {
        trie_2.insert(&line);
    }

    println!(
        "Memory usage after trie generation: {:.1}mb",
        PEAK_ALLOC.current_usage_as_mb()
    );

    let now = Instant::now();

    let result = trie_1 == trie_2;

    let elapsed = now.elapsed();
    println!("Operations time: {:.2?}", elapsed);

    assert!(result);

    println!(
        "Memory usage after trie addition: {:.1}mb\n",
        PEAK_ALLOC.current_usage_as_mb()
    );
}
