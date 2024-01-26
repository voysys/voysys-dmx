use serde::{Deserialize, Serialize};

use crate::channel::ChannelWidget;

#[derive(Serialize, Deserialize, Clone)]
pub struct Timeline {
    pub channel: ChannelWidget,
    pub gain: f32,
    pub offset: f32,
}

impl Timeline {
    pub fn new() -> Self {
        Self {
            channel: ChannelWidget::new(),
            gain: 1.0,
            offset: 0.0,
        }
    }
}
