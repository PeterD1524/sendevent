pub const EV_VERSION: i32 = 0x010001;

pub const MT_TOOL_FINGER: i32 = 0x00;
pub const MT_TOOL_PEN: i32 = 0x01;
pub const MT_TOOL_PALM: i32 = 0x02;
pub const MT_TOOL_DIAL: i32 = 0x0a;
pub const MT_TOOL_MAX: i32 = 0x0f;
pub const FF_STATUS_STOPPED: i32 = 0x00;
pub const FF_STATUS_PLAYING: i32 = 0x01;
pub const FF_STATUS_MAX: i32 = 0x01;

pub const FF_RUMBLE: i32 = 0x50;
pub const FF_PERIODIC: i32 = 0x51;
pub const FF_CONSTANT: i32 = 0x52;
pub const FF_SPRING: i32 = 0x53;
pub const FF_FRICTION: i32 = 0x54;
pub const FF_DAMPER: i32 = 0x55;
pub const FF_INERTIA: i32 = 0x56;
pub const FF_RAMP: i32 = 0x57;
pub const FF_EFFECT_MIN: i32 = FF_RUMBLE;
pub const FF_EFFECT_MAX: i32 = FF_RAMP;
pub const FF_SQUARE: i32 = 0x58;
pub const FF_TRIANGLE: i32 = 0x59;
pub const FF_SINE: i32 = 0x5a;
pub const FF_SAW_UP: i32 = 0x5b;
pub const FF_SAW_DOWN: i32 = 0x5c;
pub const FF_CUSTOM: i32 = 0x5d;
pub const FF_WAVEFORM_MIN: i32 = FF_SQUARE;
pub const FF_WAVEFORM_MAX: i32 = FF_CUSTOM;
pub const FF_GAIN: i32 = 0x60;
pub const FF_AUTOCENTER: i32 = 0x61;
pub const FF_MAX_EFFECTS: i32 = FF_GAIN;
pub const FF_MAX: i32 = 0x7f;
