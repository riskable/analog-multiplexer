//! This little module provides a place to store analog channel data (`ChannelValues`)
//! and also provides a super user-friendly channel data viewing experience
//! via `core::fmt::Display`.  Only meant for the examples but if you so desire
//! feel free to copy it into your own code for use with debugging.

// Struct for storing the state of each channel and pretty-printing it via rprintln
#[derive(Default)]
pub struct ChannelValues {
    pub ch0: u16,
    pub ch1: u16,
    pub ch2: u16,
    pub ch3: u16,
    pub ch4: u16,
    pub ch5: u16,
    pub ch6: u16,
    pub ch7: u16,
// NOTE: Channels past 7 will remain 0 if using an 8-channel multiplexer
    pub ch8: u16,
    pub ch9: u16,
    pub ch10: u16,
    pub ch11: u16,
    pub ch12: u16,
    pub ch13: u16,
    pub ch14: u16,
    pub ch15: u16,
}

impl ChannelValues {
    pub fn by_index(&self, i: u8) -> u16 {
        match i {
            0 => self.ch0,
            1 => self.ch1,
            2 => self.ch2,
            3 => self.ch3,
            4 => self.ch4,
            5 => self.ch5,
            6 => self.ch6,
            7 => self.ch7,
            8 => self.ch8,
            9 => self.ch9,
            10 => self.ch10,
            11 => self.ch11,
            12 => self.ch12,
            13 => self.ch13,
            14 => self.ch14,
            15 => self.ch15,
            _ => panic!("Invalid channel: {}", i),
        }
    }

    pub fn update_by_index(&mut self, i: u8, val: u16) {
        match i {
            0 => self.ch0 = val,
            1 => self.ch1 = val,
            2 => self.ch2 = val,
            3 => self.ch3 = val,
            4 => self.ch4 = val,
            5 => self.ch5 = val,
            6 => self.ch6 = val,
            7 => self.ch7 = val,
            8 => self.ch8 = val,
            9 => self.ch9 = val,
            10 => self.ch10 = val,
            11 => self.ch11 = val,
            12 => self.ch12 = val,
            13 => self.ch13 = val,
            14 => self.ch14 = val,
            15 => self.ch15 = val,
            _ => panic!("Invalid channel: {}", i),
        }
    }
}

// impl our super user-friendly terminal view into all channels
impl core::fmt::Display for ChannelValues {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let _ = f.write_str("\x1B[2J\x1B[0H"); // Clear the screen and move cursor to start
        let _ = f.write_str("Multiplexer Channel Values:\n");
        let _ = f.write_str("\n\x1B[1mch0\tch1\tch2\tch3\tch4\tch5\tch6\tch7\n\x1B[0m");
        for i in 0..8 {
            let _ = f.write_fmt(format_args!("{}\t", self.by_index(i))).unwrap();
        }
        let _ = f.write_str("\n");
        let _ = f.write_str("\x1B[1mch8\tch9\tch10\tch11\tch12\tch13\tch14\tch15\n\x1B[0m");
        for i in 8..16 {
            let _ = f.write_fmt(format_args!("{}\t", self.by_index(i))).unwrap();
        }
        let _ = f.write_str("\n");
        Ok(())
    }
}
