#[derive(PartialEq, Copy, Clone)]
pub enum Product {
    ArcanaLauncher,
    TwelveKnightsVigil,
}

impl Product {
    pub fn all() -> [Self; 2] {
        [Self::ArcanaLauncher, Self::TwelveKnightsVigil]
    }

    pub fn display_name(&self) -> &str {
        match *self {
            Self::ArcanaLauncher => "Arcana Launcher",
            Self::TwelveKnightsVigil => "Twelve Knights Vigil",
        }
    }
}
