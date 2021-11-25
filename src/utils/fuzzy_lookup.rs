use std::collections::HashMap;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

const MAX_AUTOCOMPLETE_RESULTS: usize = 10;

pub fn get_lookup_results(search_key: &str, elements: Vec<String>) -> Vec<String> {
    let matcher = SkimMatcherV2::default();

    let mut hits: HashMap<i64, Vec<String>> = HashMap::new();

    // TODO: This might become a bottleneck with a lot of sound files present
    for element in elements {
        if let Some(score) = matcher.fuzzy_match(&element, search_key) {
            if let Some(scored_vals) = hits.get_mut(&score) {
                scored_vals.push(element.clone());
            } else {
                hits.insert(score, vec![element.clone()]);
            }
        }
    }

    let mut score_order: Vec<i64> = hits.keys().copied().collect();
    score_order.sort_unstable();

    let mut results: Vec<String> = Vec::new();

    for score in score_order.clone() {
        if results.len() > MAX_AUTOCOMPLETE_RESULTS {
            return results;
        }

        if let Some(hit_vec) = hits.get(&score) {
            results.extend(hit_vec.clone());
        }
    }

    results
}
