//! The BM25F arithmetic of PLAN §4: field weights, length normalisation, idf and saturation.
//! Cases: SRCH-RANK-01, SRCH-WEIGHT-01

/// The saturation constant k1 (PLAN §4).
pub const K1: f64 = 1.2;

/// The length normalisation constant b (PLAN §4).
pub const B: f64 = 0.75;

/// The weight of each field in the order id, title, body (PLAN §4).
pub const WEIGHTS: [f64; 3] = [4.0, 3.0, 1.0];

/// The length normaliser of each field of each doc. `lengths` gives every doc's token count per field; the average of each field
/// is taken over the docs. A field that is empty in every doc has a normaliser of 1, and no term can reach it.
pub fn norms(lengths: &[[f64; 3]]) -> Vec<[f64; 3]> {
    let docs = lengths.len() as f64;
    let mut averages = [0.0; 3];
    for length in lengths {
        for (average, len) in averages.iter_mut().zip(length) {
            *average += len / docs;
        }
    }
    lengths
        .iter()
        .map(|length| std::array::from_fn(|field| normaliser(length[field], averages[field])))
        .collect()
}

/// The normaliser of one field of one doc: `1 / (1 - b + b * len / avg)`, or 1 when the field's average length is zero.
fn normaliser(len: f64, average: f64) -> f64 {
    if average <= 0.0 {
        1.0
    } else {
        1.0 / ((1.0 - B) + B * len / average)
    }
}

/// The weighted frequency of a term in one doc: each field's count times its weight and its normaliser, summed.
pub fn weight(tf: [u32; 3], norm: [f64; 3]) -> f64 {
    WEIGHTS
        .iter()
        .zip(tf)
        .zip(norm)
        .map(|((weight, count), norm)| weight * f64::from(count) * norm)
        .sum()
}

/// The inverse document frequency of a term that is in `df` of `docs` docs: `ln(1 + (N - n + 0.5) / (n + 0.5))`.
pub fn idf(docs: usize, df: usize) -> f64 {
    let (docs, df) = (docs as f64, df as f64);
    (1.0 + (docs - df + 0.5) / (df + 0.5)).ln()
}

/// The saturation of a weighted frequency `s`: `s / (k1 + s)`.
pub fn saturate(s: f64) -> f64 {
    s / (K1 + s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_term_in_every_doc_has_a_small_idf_and_a_rare_one_a_large_idf() {
        assert!(idf(4, 4) < idf(4, 1));
        assert!(idf(4, 4) > 0.0);
    }

    #[test]
    fn a_longer_doc_has_a_smaller_weight_for_the_same_count() {
        let short = norms(&[[1.0, 1.0, 1.0], [1.0, 1.0, 1.0]]);
        let long = norms(&[[1.0, 1.0, 9.0], [1.0, 1.0, 1.0]]);
        assert!(weight([0, 0, 1], long[0]) < weight([0, 0, 1], short[0]));
    }
}
