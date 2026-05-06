use rand::RngExt;

pub fn random_string(len: usize) -> String {
    rand::rngs::ThreadRng::default()
        .sample_iter(rand::distr::Alphabetic)
        .take(len)
        .map(char::from)
        .collect::<String>()
}
