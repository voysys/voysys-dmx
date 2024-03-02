use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Default)]
pub enum PxHex5Strobe {
    #[default]
    NoStrobe,
    Strobe,
    StrobePuls,
    StrobeRandom,
}

impl PxHex5Strobe {
    pub fn name(&self) -> &str {
        match self {
            PxHex5Strobe::NoStrobe => "Strobe Off",
            PxHex5Strobe::Strobe => "Strobe",
            PxHex5Strobe::StrobePuls => "Strobe Puls",
            PxHex5Strobe::StrobeRandom => "Strobe Random",
        }
    }
}

impl std::fmt::Display for PxHex5Strobe {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Serialize, Deserialize, PartialEq, Default)]
pub struct PxHex5 {
    pub strobe: PxHex5Strobe,
    pub strobe_value: u8,
}

impl PxHex5 {
    pub fn to_dmx_value(&self) -> u8 {
        match self.strobe {
            PxHex5Strobe::NoStrobe => 0,
            PxHex5Strobe::Strobe => 64 + self.strobe_value,
            PxHex5Strobe::StrobePuls => 128 + self.strobe_value,
            PxHex5Strobe::StrobeRandom => 192 + self.strobe_value,
        }
    }
}
