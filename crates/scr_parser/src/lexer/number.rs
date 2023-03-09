pub fn parse(src: &str) -> Result<f64, &'static str> {
    src.parse::<f64>().map_err(|_| "invalid float")
}
