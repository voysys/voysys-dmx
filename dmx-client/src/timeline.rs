use serde::{Deserialize, Serialize};

use crate::channel::ChannelWidget;

#[derive(Serialize, Deserialize, Clone)]
pub struct Timeline {
    pub id: i8,
    pub channel: ChannelWidget,
    pub gain: f32,
    pub offset: f32,
}

impl Timeline {
    pub fn new(id: i8) -> Self {
        Self {
            id,
            channel: ChannelWidget::new(),
            gain: 1.0,
            offset: 0.0,
        }
    }
}
