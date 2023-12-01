use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DmxMessage {
    pub buffer: Vec<u8>,
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct DmxColor {
    pub rgb: [u8; 3],
    pub white: u8,
    pub amber: u8,
    pub uv: u8,
}

impl DmxColor {
    pub fn dmx(self) -> [u8; 12] {
        [
            self.rgb[0],
            self.rgb[1],
            self.rgb[2],
            self.white,
            self.amber,
            self.uv,
            0xff,
            0xff,
            0x00,
            0x00,
            0x00,
            0x00,
        ]
    }
}
