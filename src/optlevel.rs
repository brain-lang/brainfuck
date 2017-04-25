use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub enum OptimizationLevel {
    // Do not optimize
    Off,
    // Optimize for speed
    Speed,
}

impl FromStr for OptimizationLevel {
    type Err = ();

    fn from_str(val: &str) -> Result<Self, Self::Err> {
        match val {
            "0" => Ok(OptimizationLevel::Off),
            "1" => Ok(OptimizationLevel::Speed),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str() {
        let opt: OptimizationLevel = "0".parse().unwrap();
        assert_eq!(opt, OptimizationLevel::Off);
        let opt: OptimizationLevel = "1".parse().unwrap();
        assert_eq!(opt, OptimizationLevel::Speed);

        assert!("2".parse::<OptimizationLevel>().is_err());
    }
}
