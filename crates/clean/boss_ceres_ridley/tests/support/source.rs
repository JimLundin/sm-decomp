use serde::Deserialize;

#[derive(Deserialize)]
#[serde(try_from = "String")]
pub enum Source {
    Constructed,
    Recorded,
    Oracle,
}

impl TryFrom<String> for Source {
    type Error = &'static str;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        match text.as_str() {
            "constructed" => Ok(Self::Constructed),
            "recorded" => Ok(Self::Recorded),
            "oracle" => Ok(Self::Oracle),
            _ if text.split_once(':').is_some_and(|(route, cut)| {
                (route.starts_with("chain-") || route == "rS1") && cut.parse::<u32>().is_ok()
            }) =>
            {
                Ok(Self::Recorded)
            }
            _ => Err("unknown vector source"),
        }
    }
}

impl Source {
    pub fn check<T>(&self, run: impl FnOnce() -> T) -> T {
        match self {
            Self::Constructed | Self::Recorded | Self::Oracle => run(),
        }
    }
}
