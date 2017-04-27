use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub enum DebugFormat {
    /// Human readable text format
    Text,
    /// Machine readable JSON format
    Json,
}

impl FromStr for DebugFormat {
    type Err = ();

    fn from_str(val: &str) -> Result<Self, Self::Err> {
        match val {
            "text" => Ok(DebugFormat::Text),
            "json" => Ok(DebugFormat::Json),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str() {
        let opt: DebugFormat = "text".parse().unwrap();
        assert_eq!(opt, DebugFormat::Text);
        let opt: DebugFormat = "json".parse().unwrap();
        assert_eq!(opt, DebugFormat::Json);

        assert!("foo".parse::<DebugFormat>().is_err());
    }
}
