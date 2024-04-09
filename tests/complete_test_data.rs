use basic_trie::DataTrie;
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

    while number_of_words != x {
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
fn overall_data() {
    let number_of_words = 500_000;
    let word_length = 15;

    let big_data = generate_random_lines(number_of_words, word_length);

    println!(
        "Memory usage after loading words: {:.1}mb\n",
        PEAK_ALLOC.current_usage_as_mb()
    );

    let mut data_trie = DataTrie::new();

    for word in big_data.big_data {
        data_trie.insert(&word, 1000);
    }

    println!(
        "Memory usage after data trie generation: {:.1}mb",
        PEAK_ALLOC.current_usage_as_mb()
    );

    let now = Instant::now();

    assert_eq!(data_trie.len(), big_data.number_of_words);
    assert_eq!(
        word_length,
        data_trie.get_longest()[0].graphemes(true).count()
    );
    assert_eq!(
        word_length,
        data_trie.get_shortest()[0].graphemes(true).count()
    );

    for (first_letter, count) in big_data.first_letter_histogram.iter() {
        assert_eq!(*count, data_trie.len_prefix(first_letter));
    }

    for (first_letter, count) in big_data.first_letter_histogram.iter() {
        assert_eq!(
            vec![1000; *count],
            data_trie.remove_prefix(first_letter).unwrap()
        );
    }

    assert!(data_trie.is_empty());

    let elapsed = now.elapsed();
    println!("Operations time: {:.2?}", elapsed);
}

#[test]
fn clearing_data() {
    let number_of_words = 500_000;
    let word_length = 15;

    let big_data = generate_random_lines(number_of_words, word_length);

    println!(
        "Memory usage after loading words: {:.1}mb\n",
        PEAK_ALLOC.current_usage_as_mb()
    );

    let mut data_trie = DataTrie::new();

    for word in big_data.big_data {
        data_trie.insert(&word, 0);
    }

    println!(
        "Memory usage after data trie generation: {:.1}mb\n",
        PEAK_ALLOC.current_usage_as_mb()
    );

    let now = Instant::now();

    data_trie.clear();
    assert!(data_trie.is_empty());

    let elapsed = now.elapsed();
    println!("Operations time: {:.2?}\n", elapsed);

    println!(
        "Memory usage after data trie cleanup: {:.1}mb\n",
        PEAK_ALLOC.current_usage_as_mb()
    );
}

#[test]
fn add_op_data_1() {
    let number_of_words = 100_000;
    let word_length = 15;

    let big_data = generate_random_lines(number_of_words, word_length);

    let mut data_trie_0 = DataTrie::new();
    let mut data_trie_1 = DataTrie::new();
    let mut data_trie_2 = DataTrie::new();

    println!(
        "Memory usage after loading words: {:.1}mb\n",
        PEAK_ALLOC.current_usage_as_mb()
    );

    for line in big_data.big_data.iter() {
        data_trie_0.insert(line, line.as_str());
    }

    for line in big_data
        .big_data
        .iter()
        .rev()
        .skip(big_data.number_of_words / 2)
    {
        data_trie_1.insert(line, line.as_str());
    }

    for line in big_data.big_data.iter().skip(big_data.number_of_words / 2) {
        data_trie_2.insert(line, line.as_str());
    }

    println!(
        "Memory usage after data trie generation: {:.1}mb",
        PEAK_ALLOC.current_usage_as_mb()
    );

    let now = Instant::now();

    data_trie_1 += data_trie_2;

    let elapsed = now.elapsed();
    println!("Operations time: {:.2?}", elapsed);

    let t1_words = data_trie_1.get_all();
    let correct_words = data_trie_0.get_all();

    let item_set: HashSet<_> = t1_words.iter().collect();
    let only_in_correct: Vec<_> = correct_words
        .into_iter()
        .filter(|item| !item_set.contains(item))
        .collect();

    assert_eq!(only_in_correct.len(), 0);
    println!("{}", big_data.number_of_words);
    assert!(data_trie_0 == data_trie_1);

    println!(
        "Memory usage after data trie addition: {:.1}mb\n",
        PEAK_ALLOC.current_usage_as_mb()
    );
}

#[test]
fn add_op_data_2() {
    let number_of_words = 100_000;
    let word_length = 15;

    let big_data = generate_random_lines(number_of_words, word_length);

    let mut data_trie_0 = DataTrie::new();
    let mut data_trie_1 = DataTrie::new();
    let mut data_trie_2 = DataTrie::new();

    println!(
        "Memory usage after loading words: {:.1}mb\n",
        PEAK_ALLOC.current_usage_as_mb()
    );

    for line in big_data.big_data.iter() {
        data_trie_0.insert(&line, line.as_str());
    }

    for line in big_data.big_data.iter().rev().skip(number_of_words / 2) {
        data_trie_1.insert(&line, line.as_str());
    }

    for line in big_data.big_data.iter().skip(number_of_words / 2) {
        data_trie_2.insert(&line, line.as_str());
    }

    println!(
        "Memory usage after data trie generation: {:.1}mb",
        PEAK_ALLOC.current_usage_as_mb()
    );

    let now = Instant::now();

    let data_trie_3 = data_trie_1 + data_trie_2;
    let elapsed = now.elapsed();
    println!("Operations time: {:.2?}", elapsed);

    let t3_words = data_trie_3.get_all();
    let correct_words = data_trie_0.get_all();

    let item_set: HashSet<_> = t3_words.iter().collect();
    let only_in_correct: Vec<_> = correct_words
        .into_iter()
        .filter(|item| !item_set.contains(item))
        .collect();

    assert_eq!(only_in_correct.len(), 0);
    assert!(data_trie_0 == data_trie_3);

    println!(
        "Memory usage after data trie addition: {:.1}mb\n",
        PEAK_ALLOC.current_usage_as_mb()
    );
}

#[test]
fn equals_data() {
    let number_of_words = 100_000;
    let word_length = 15;

    let big_data = generate_random_lines(number_of_words, word_length);

    let mut data_trie_1 = DataTrie::new();
    let mut data_trie_2 = DataTrie::new();

    println!(
        "Memory usage after loading words: {:.1}mb\n",
        PEAK_ALLOC.current_usage_as_mb()
    );

    for line in big_data.big_data.iter() {
        data_trie_1.insert(&line, line.as_str());
    }

    for line in big_data.big_data.iter() {
        data_trie_2.insert(&line, line.as_str());
    }

    println!(
        "Memory usage after data trie generation: {:.1}mb",
        PEAK_ALLOC.current_usage_as_mb()
    );

    let now = Instant::now();

    let result = data_trie_1 == data_trie_2;

    let elapsed = now.elapsed();
    println!("Operations time: {:.2?}", elapsed);

    assert!(result);

    println!(
        "Memory usage after data trie addition: {:.1}mb\n",
        PEAK_ALLOC.current_usage_as_mb()
    );
}
