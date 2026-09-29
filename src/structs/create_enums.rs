
#[derive(Debug)]
pub enum Languages {
    Python,
    JS,
    TS,
    Rust,
    Go,
    Java,
    C,
}
impl TryFrom<usize> for Languages {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Languages::Python),
            1 => Ok(Languages::JS),
            2 => Ok(Languages::TS),
            3 => Ok(Languages::Rust),
            4 => Ok(Languages::Go),
            5 => Ok(Languages::Java),
            6 => Ok(Languages::C),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum ExtraChoices {
    Docker,
    Readme,
}
impl TryFrom<usize> for ExtraChoices {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ExtraChoices::Docker),
            1 => Ok(ExtraChoices::Readme),
            _ => Err(()),
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum Step {
    ChooseName = 0,
    ChooseProjectType,
    ChooseLanguage,
    ChooseExtras,
    Confirmation,
}