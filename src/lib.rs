//! # Basic Trie
//!
//! The trie data structure is used for quick access to words and
//! data that should be associated with them.
//!
//! **Basic Trie** is implemented as a tree where each node holds a single character
//! that could point at any other character thus allowing insertion of arbitrary words.
//! Each node also holds a vector of the data that is associated with it.
//!
//! For example, when inserting a whole book in the trie, you could insert every word with
//! the corresponding page number it's on. Later when searching for the word, you could get all
//! the pages the word is on with no added performance cost.
//!
//! ## Features
//! - insertion / removal of words
//! - finding words based on prefix
//! - finding data of words based on exact match or prefix
//! - longest / shortest words in the trie
//! - number of complete words in the trie
//!
//! ## Optional features
//! - unicode support via the 'unicode' feature with the 'unicode-segmentation' crate (enabled by default)
//!
//! ## Dependencies
//! - unicode-segmentation (enabled by default)
//!
//! ## License
//!
//! The software is licensed under the MIT license.

mod trie;

pub use trie::Trie;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_words() {
        let found_words_correct = vec![
            String::from("word1"),
            String::from("word2"),
            String::from("word3"),
        ];

        let mut trie = Trie::new();

        trie.insert("word1", "data1");
        trie.insert("word2", "data2");
        trie.insert("word3", "data3");

        let mut found_words = trie.find_words("word").unwrap();
        found_words.sort();
        assert_eq!(found_words, found_words_correct);
    }

    #[test]
    fn find_words_num() {
        let found_words_correct = vec![
            String::from("word1"),
            String::from("word2"),
            String::from("word3"),
        ];

        let mut trie = Trie::new();

        trie.insert("word1", 1);
        trie.insert("word2", 2);
        trie.insert("word3", 3);

        let mut found_words = trie.find_words("word").unwrap();
        found_words.sort();
        assert_eq!(found_words, found_words_correct);
    }

    #[test]
    fn find_data_soft_match() {
        let found_data_correct = vec![&1, &2, &3];

        let mut trie = Trie::new();

        trie.insert("word1", 1);
        trie.insert("word2", 2);
        trie.insert("word3", 3);

        let mut found_data = trie.find_data_of_word("word", true).unwrap();
        found_data.sort();
        assert_eq!(found_data, found_data_correct);
    }

    #[test]
    fn find_str_data_soft_match() {
        let found_data_correct = vec![&"data1", &"data2", &"data3"];

        let mut trie = Trie::new();

        trie.insert("word1", "data1");
        trie.insert("word2", "data2");
        trie.insert("word3", "data3");

        let mut found_data = trie.find_data_of_word("word", true).unwrap();
        found_data.sort();
        assert_eq!(found_data, found_data_correct);
    }

    #[test]
    fn find_data_hard_match() {
        let found_data_correct = vec![&1];

        let mut trie = Trie::new();

        trie.insert("word1", 1);
        trie.insert("word2", 2);
        trie.insert("word3", 3);

        let mut found_data = trie.find_data_of_word("word1", false).unwrap();
        found_data.sort();
        assert_eq!(found_data, found_data_correct);
    }

    #[test]
    fn find_data_hard_match_not_found() {
        let found_data_correct = None;

        let mut trie = Trie::new();

        trie.insert("word1", 1);
        trie.insert("word2", 2);
        trie.insert("word3", 3);

        let found_data = trie.find_data_of_word("word", false);

        assert_eq!(found_data, found_data_correct);
    }

    #[test]
    fn longest_word() {
        let mut trie = Trie::new();

        trie.insert("a", 5);
        assert_eq!(trie.longest_words(), vec![String::from("a")]);

        trie.insert("aa", 5);
        assert_eq!(trie.longest_words(), vec![String::from("aa")]);

        trie.insert("aaa", 5);
        assert_eq!(trie.longest_words(), vec![String::from("aaa")]);

        trie.insert("aaaa", 5);
        assert_eq!(trie.longest_words(), vec![String::from("aaaa")]);

        trie.insert("a", 5);
        assert_eq!(trie.longest_words(), vec![String::from("aaaa")]);
    }

    #[test]
    fn multiple_longest_words() {
        let mut trie = Trie::new();

        trie.insert("abba", 5);
        trie.insert("cddc", 5);

        let mut found_words = trie.longest_words();
        found_words.sort();

        assert_eq!(
            vec![String::from("abba"), String::from("cddc")],
            found_words
        );
    }

    #[test]
    fn shortest_word() {
        let mut trie = Trie::new();

        trie.insert("a", 5);
        assert_eq!(trie.shortest_words(), vec![String::from("a")]);

        trie.insert("aa", 5);
        assert_eq!(trie.shortest_words(), vec![String::from("a")]);

        trie.insert("aaa", 5);
        assert_eq!(trie.shortest_words(), vec![String::from("a")]);

        trie.insert("aaaa", 5);
        assert_eq!(trie.shortest_words(), vec![String::from("a")]);

        trie.insert("a", 5);
        assert_eq!(trie.shortest_words(), vec![String::from("a")]);
    }

    #[test]
    fn multiple_shortest_words() {
        let mut trie = Trie::new();

        trie.insert("aaa", 5);
        trie.insert("aaaa", 5);
        trie.insert("aa", 5);
        trie.insert("bb", 5);

        let mut found_words = trie.shortest_words();
        found_words.sort();

        assert_eq!(vec![String::from("aa"), String::from("bb")], found_words);
    }

    #[test]
    fn number_of_words() {
        let mut trie = Trie::new();

        trie.insert("a", 1);
        trie.insert("b", 1);
        trie.insert("c", 1);
        trie.insert("d", 1);

        assert_eq!(4, trie.number_of_words());
    }

    #[test]
    fn same_word_twice() {
        let mut trie = Trie::new();

        trie.insert("twice", 5);
        trie.insert("twice", 5);

        assert_eq!(vec!["twice"], trie.find_words("twice").unwrap());
    }

    #[test]
    fn same_word_twice_different_data() {
        let mut trie = Trie::new();

        trie.insert("twice", 5);
        trie.insert("twice", 3);

        assert_eq!(vec![&5, &3], trie.find_data_of_word("twice", true).unwrap());
    }

    #[test]
    fn clear_word_data() {
        let mut trie = Trie::new();

        trie.insert("twice", 5);
        trie.clear_word_data("twice");
        trie.insert("twice", 3);

        assert_eq!(vec![&3], trie.find_data_of_word("twice", true).unwrap());
    }

    #[test]
    fn all_words() {
        let mut trie = Trie::new();

        trie.insert("a", 5);
        trie.insert("ab", 5);
        trie.insert("abc", 5);
        trie.insert("abcd", 5);

        let all_words = vec![
            String::from("a"),
            String::from("ab"),
            String::from("abc"),
            String::from("abcd"),
        ];

        assert_eq!(all_words, trie.all_words().unwrap())
    }

    #[test]
    fn remove_word1() {
        let mut trie = Trie::new();

        trie.insert("a", 5);
        trie.insert("ab", 5);
        trie.insert("abc", 5);
        trie.insert("abcd", 5);

        trie.remove_word("a");

        let all_words = vec![
            String::from("ab"),
            String::from("abc"),
            String::from("abcd"),
        ];

        assert_eq!(all_words, trie.all_words().unwrap())
    }

    #[test]
    fn remove_word_final() {
        let mut trie = Trie::new();

        trie.insert("a", 5);
        trie.insert("ab", 5);
        trie.insert("abc", 5);
        trie.insert("abcd", 5);

        trie.remove_word("abcd");

        let all_correct_words = vec![String::from("a"), String::from("ab"), String::from("abc")];

        let mut all_words = trie.all_words().unwrap();
        all_words.sort();

        assert_eq!(all_correct_words, all_words);
    }

    #[test]
    fn remove_word_2() {
        let mut trie = Trie::new();

        trie.insert("a", 5);
        trie.insert("ab", 5);
        trie.insert("abc", 5);
        trie.insert("abcd", 5);

        trie.remove_word("abc");

        let all_correct_words = vec![String::from("a"), String::from("ab"), String::from("abcd")];

        let mut all_words = trie.all_words().unwrap();
        all_words.sort();

        assert_eq!(all_correct_words, all_words);
        assert_eq!(vec![&5, &5, &5], trie.find_data_of_word("a", true).unwrap());
    }

    #[test]
    fn remove_word_3() {
        let mut trie = Trie::new();

        trie.insert("eat", 5);
        trie.insert("eating", 5);
        trie.insert("eats", 5);
        trie.insert("eatings", 5);

        trie.remove_word("eating");

        let all_correct_words = vec![
            String::from("eat"),
            String::from("eatings"),
            String::from("eats"),
        ];

        let mut all_words = trie.all_words().unwrap();
        all_words.sort();

        assert_eq!(all_correct_words, all_words);
    }

    #[test]
    fn remove_word_4() {
        let mut trie = Trie::new();

        trie.insert("eat", 5);
        trie.insert("eating", 5);
        trie.insert("eats", 5);
        trie.insert("eatings", 5);

        trie.remove_word("eatings");

        let all_correct_words = vec![
            String::from("eat"),
            String::from("eating"),
            String::from("eats"),
        ];

        let mut all_words = trie.all_words().unwrap();
        all_words.sort();

        assert_eq!(all_correct_words, all_words);
    }

    #[test]
    fn remove_word_5() {
        let mut trie = Trie::new();

        trie.insert("eat", 5);
        trie.insert("eating", 5);
        trie.insert("eats", 5);
        trie.insert("eatings", 5);

        trie.remove_word("eatin");

        let all_correct_words = vec![
            String::from("eat"),
            String::from("eating"),
            String::from("eatings"),
            String::from("eats"),
        ];

        let mut all_words = trie.all_words().unwrap();
        all_words.sort();

        assert_eq!(all_correct_words, all_words);
    }

    #[test]
    fn remove_prefix() {
        let mut trie = Trie::new();

        trie.insert("eat", 5);
        trie.insert("eating", 5);
        trie.insert("eats", 5);
        trie.insert("eatings", 5);

        trie.remove_words_from_prefix("e");

        assert_eq!(None, trie.find_data_of_word("", true));
    }

    #[cfg(feature = "unicode")]
    #[test]
    fn unicode() {
        let mut trie = Trie::new();

        trie.insert("а", 5);
        trie.insert("аб", 5);
        trie.insert("абц", 5);
        trie.insert("абцд", 5);

        let all_words = vec![
            String::from("а"),
            String::from("аб"),
            String::from("абц"),
            String::from("абцд"),
        ];

        assert_eq!(all_words, trie.all_words().unwrap())
    }

    #[cfg(feature = "unicode")]
    #[test]
    fn unicode2() {
        let mut trie = Trie::new();

        trie.insert("а", 5);
        trie.insert("аб", 5);
        trie.insert("абц", 5);
        trie.insert("абцд", 5);

        let all_data = vec![&5, &5, &5, &5];

        assert_eq!(all_data, trie.find_data_of_word("а", true).unwrap())
    }
}
