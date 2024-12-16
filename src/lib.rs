use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::time::{Duration, SystemTime};
use std::{iter, num, str, thread};

use linux::input_event_codes;

mod gen;
pub mod linux;

#[derive(Debug)]
struct TimeVal {
    sec: i64,
    usec: i64,
}

impl TimeVal {
    fn to_duration(&self) -> Duration {
        Duration::new(
            self.sec.try_into().unwrap(),
            (self.usec * 1000).try_into().unwrap(),
        )
    }
}

#[derive(Debug)]
struct InputEvent {
    time: TimeVal,
    r#type: u16,
    code: u16,
    value: i32,
}

impl InputEvent {
    fn to_ne_bytes(&self) -> Vec<u8> {
        let buf: &[&[u8]] = &[
            &self.time.sec.to_ne_bytes(),
            &self.time.usec.to_ne_bytes(),
            &self.r#type.to_ne_bytes(),
            &self.code.to_ne_bytes(),
            &self.value.to_ne_bytes(),
        ];
        buf.concat()
    }
}

fn parse_event(line: &str, options: &Options) -> Result<(Option<String>, InputEvent), Error> {
    let line_saved = line;
    let (line, sec, usec) = if options.get_time {
        let bytes = line.as_bytes();
        if let Some(&c) = bytes.get(0) {
            if c != b'[' {
                return Err(Error::Format(format!(
                    "missing `[` for line: {:?}",
                    line_saved
                )));
            }
        } else {
            return Err(Error::Format("empty line".to_string()));
        }
        let line = match str::from_utf8(&bytes[1..]) {
            Ok(line) => line,
            Err(error) => return Err(Error::Utf8(error)),
        };
        let (time, line) = if let Some((first, second)) = line.split_once("] ") {
            (first, second)
        } else {
            return Err(Error::Format(format!(
                "missing `] ` for line: {:?}",
                line_saved
            )));
        };
        if let Some((first, second)) = time.split_once('.') {
            (
                line,
                match first.trim_start().parse() {
                    Ok(first) => first,
                    Err(error) => {
                        return Err(Error::ParseInt(
                            error,
                            format!("parsing time sec field for line: {:?}", line_saved),
                        ))
                    }
                },
                match second.parse() {
                    Ok(second) => second,
                    Err(error) => {
                        return Err(Error::ParseInt(
                            error,
                            format!("parsing time usec field for line: {:?}", line_saved),
                        ))
                    }
                },
            )
        } else {
            return Err(Error::Format(format!(
                "missing `.` in timestamp field for line: {:?}",
                line_saved
            )));
        }
    } else {
        (line, 0, 0)
    };

    let (line, device) = if options.print_device {
        if let Some((first, second)) = line.split_once(": ") {
            (second, Some(first.to_string()))
        } else {
            return Err(Error::Format(format!(
                "missing device field for line: {:?}",
                line_saved
            )));
        }
    } else {
        (line, None)
    };

    let mut splits = line.split(' ').filter(|s| !s.is_empty());
    let r#type = if let Some(s) = splits.next() {
        match gen::get_type_value(s) {
            Ok(value) => value,
            Err(error) => {
                return Err(Error::ParseInt(
                    error,
                    format!("parsing type field for line: {:?}", line_saved),
                ))
            }
        }
    } else {
        return Err(Error::Format(format!(
            "missing type field for line: {:?}",
            line_saved
        )));
    };

    let code = if let Some(s) = splits.next() {
        match gen::get_code_value(r#type, s) {
            Ok(value) => value,
            Err(error) => {
                return Err(Error::ParseInt(
                    error,
                    format!("parsing code field for line: {:?}", line_saved),
                ))
            }
        }
    } else {
        return Err(Error::Format(format!(
            "missing code field for line: {:?}",
            line_saved
        )));
    };

    let value = if let Some(s) = splits.next() {
        match gen::get_value_value(r#type, code, s) {
            Ok(value) => value,
            Err(error) => {
                return Err(Error::ParseInt(
                    error,
                    format!("parsing value field for line: {:?}", line_saved),
                ))
            }
        }
    } else {
        return Err(Error::Format(format!(
            "missing value field for line: {:?}",
            line_saved
        )));
    };

    Ok((
        device,
        InputEvent {
            time: TimeVal { sec, usec },
            r#type,
            code,
            value,
        },
    ))
}

fn write_event(device: &mut impl Write, event: &InputEvent) -> Result<(), io::Error> {
    device.write_all(&event.to_ne_bytes())
}

fn get_options(line: &str) -> Result<Options, Error> {
    let get_time = if let Some(&c) = line.as_bytes().get(0) {
        c == b'['
    } else {
        return Err(Error::Format("empty line".to_string()));
    };

    let line = if get_time {
        if let Some((_, second)) = line.split_once("] ") {
            second
        } else {
            return Err(Error::Format("misssing `]`".to_string()));
        }
    } else {
        line
    };

    let print_device = if let Some(_) = line.split_once(": ") {
        true
    } else {
        false
    };

    Ok(Options {
        get_time,
        print_device,
    })
}

#[derive(Debug)]
struct Options {
    get_time: bool,
    print_device: bool,
}

#[derive(Debug)]
enum Error {
    Format(String),
    IO(io::Error),
    Utf8(str::Utf8Error),
    ParseInt(num::ParseIntError, String),
}

fn parse_all(
    reader: &mut impl BufRead,
) -> Box<dyn Iterator<Item = Result<(Option<String>, InputEvent), Error>> + '_> {
    let mut lines = reader.lines();
    let line = if let Some(result) = lines.next() {
        match result {
            Ok(line) => line,
            Err(error) => return Box::new(iter::once(Err(Error::IO(error)))),
        }
    } else {
        return Box::new(iter::empty());
    };
    let options = match get_options(&line) {
        Ok(options) => options,
        Err(error) => return Box::new(iter::once(Err(error))),
    };
    Box::new(lines.map(move |result| match result {
        Ok(line) => parse_event(&line, &options),
        Err(error) => Err(Error::IO(error)),
    }))
}

pub fn send_events_from_reader(reader: &mut impl BufRead, device: Option<&str>) {
    fn preprocess_result<'a>(
        device: &Option<&str>,
        opened_devices: &'a mut HashMap<String, File>,
        result: Result<(Option<String>, InputEvent), Error>,
    ) -> Option<(&'a mut File, InputEvent)> {
        let (device, event) = match result {
            Ok((option, event)) => (
                if let Some(device) = option {
                    device
                } else {
                    device.unwrap().to_string()
                },
                event,
            ),
            Err(error) => panic!("{:?}", error),
        };
        let event = InputEvent {
            time: TimeVal { sec: 0, usec: 0 },
            ..event
        };
        let device = opened_devices
            .entry(device.clone())
            .or_insert_with(|| File::options().write(true).open(&device).unwrap());
        if !(i32::try_from(event.r#type).unwrap() == input_event_codes::EV_SYN
            && i32::try_from(event.code).unwrap() == input_event_codes::SYN_REPORT)
        {
            write_event(device, &event).unwrap();
            return None;
        }
        Some((device, event))
    }

    let mut opened_devices = HashMap::new();

    let mut iterator = parse_all(reader);
    if let Some((base_system_time, base_event_time)) = loop {
        if let Some(result) = iterator.next() {
            if let Some((device, event)) = preprocess_result(&device, &mut opened_devices, result) {
                let base_event_time = event.time.to_duration();
                let base_system_time = SystemTime::now();
                write_event(device, &event).unwrap();
                break Some((base_system_time, base_event_time));
            }
            continue;
        }
        break None;
    } {
        for result in iterator {
            if let Some((device, event)) = preprocess_result(&device, &mut opened_devices, result) {
                let current_event_time = event.time.to_duration();
                let eta = current_event_time - base_event_time;
                let now = base_system_time.elapsed().unwrap();
                let delay = eta.saturating_sub(now);
                thread::sleep(delay);
                write_event(device, &event).unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        linux::input_event_codes, parse_event, write_event, Error, InputEvent, Options, TimeVal,
    };
    use std::time::Duration;

    #[test]
    fn time_val_to_duration() {
        assert_eq!(
            TimeVal { sec: 0, usec: 0 }.to_duration(),
            Duration::new(0, 0)
        );
        assert_eq!(
            super::TimeVal { sec: 1, usec: 1 }.to_duration(),
            Duration::new(1, 1000)
        );
        assert_eq!(
            super::TimeVal {
                sec: i64::MAX,
                usec: 0
            }
            .to_duration(),
            Duration::new(i64::MAX.try_into().unwrap(), 0)
        );
        assert_eq!(
            super::TimeVal {
                sec: i64::MAX,
                usec: 4294967
            }
            .to_duration(),
            Duration::new(i64::MAX.try_into().unwrap(), 4294967000)
        );
    }

    #[test]
    fn test_parse_event() {
        macro_rules! run {
            ($pattern:pat, $line:expr, $options:expr) => {
                run!($pattern, $line, $options, {});
            };
            ($pattern:pat, $line:expr, $options:expr, $statement:expr) => {
                if let $pattern = parse_event($line, &$options) {
                    $statement;
                } else {
                    assert!(false);
                }
            };
        }
        run!(
            Err(Error::Format(..)),
            "",
            Options {
                get_time: false,
                print_device: false,
            }
        );
        run!(
            Err(Error::Format(..)),
            "",
            Options {
                get_time: false,
                print_device: true,
            }
        );
        run!(
            Err(Error::Format(..)),
            "",
            Options {
                get_time: true,
                print_device: false,
            }
        );
        run!(
            Err(Error::Format(..)),
            "",
            Options {
                get_time: true,
                print_device: true,
            }
        );
        run!(
            Err(Error::Format(..)),
            "[",
            Options {
                get_time: true,
                print_device: false,
            }
        );
        run!(
            Err(Error::Format(..)),
            "[] ",
            Options {
                get_time: true,
                print_device: false,
            }
        );
        run!(
            Err(Error::ParseInt(..)),
            "[.] ",
            Options {
                get_time: true,
                print_device: false,
            }
        );
        run!(
            Err(Error::ParseInt(..)),
            "[0.] ",
            Options {
                get_time: true,
                print_device: false,
            }
        );
        run!(
            Err(Error::Format(..)),
            "[0.0] ",
            Options {
                get_time: true,
                print_device: false,
            }
        );
        run!(
            Err(Error::Format(..)),
            ": ",
            Options {
                get_time: false,
                print_device: true,
            }
        );
        run!(
            Err(Error::ParseInt(..)),
            "-1",
            Options {
                get_time: false,
                print_device: false,
            }
        );
        run!(
            Err(Error::Format(..)),
            "0",
            Options {
                get_time: false,
                print_device: false,
            }
        );
        run!(
            Err(Error::ParseInt(..)),
            "0 -1",
            Options {
                get_time: false,
                print_device: false,
            }
        );
        run!(
            Err(Error::ParseInt(..)),
            "0 -1",
            Options {
                get_time: false,
                print_device: false,
            }
        );
        run!(
            Err(Error::Format(..)),
            "0 0",
            Options {
                get_time: false,
                print_device: false,
            }
        );
        run!(
            Err(Error::ParseInt(..)),
            "0 0 -1",
            Options {
                get_time: false,
                print_device: false,
            }
        );
        run!(
            Ok((
                None,
                InputEvent {
                    time: TimeVal { sec: 0, usec: 0 },
                    r#type: 0,
                    code: 0,
                    value: 0
                }
            )),
            "0 0 0",
            Options {
                get_time: false,
                print_device: false,
            }
        );
        run!(
            Ok((
                None,
                InputEvent {
                    time: TimeVal { sec: 1, usec: 1 },
                    r#type: 1,
                    code: 1,
                    value: 1
                }
            )),
            "[1.1] 1 1 1",
            Options {
                get_time: true,
                print_device: false,
            }
        );
        run!(
            Ok((
                Some(device),
                InputEvent {
                    time: TimeVal { sec: 0, usec: 0 },
                    r#type: 0,
                    code: 0,
                    value: 0
                }
            )),
            "[0.0] : 0 0 0",
            Options {
                get_time: true,
                print_device: true,
            },
            assert_eq!(device, "")
        );
        run!(
            Ok((
                Some(device),
                InputEvent {
                    time: TimeVal { sec: 0, usec: 0 },
                    r#type: 0,
                    code: 0,
                    value: 0
                }
            )),
            "[0.0] /dev/input/event0: 0 0 0",
            Options {
                get_time: true,
                print_device: true,
            },
            assert_eq!(device, "/dev/input/event0")
        );
        run!(
            Ok((
                Some(device),
                InputEvent {
                    time: TimeVal { sec: 0, usec: 0 },
                    r#type,
                    code,
                    value
                }
            )),
            "[0.0] /dev/input/event0: EV_SYN SYN_REPORT 0",
            Options {
                get_time: true,
                print_device: true,
            },
            {
                assert_eq!(device, "/dev/input/event0");
                assert_eq!(r#type, input_event_codes::EV_SYN.try_into().unwrap());
                assert_eq!(code, input_event_codes::SYN_REPORT.try_into().unwrap());
                assert_eq!(value, 0);
            }
        );
        run!(
            Ok((
                Some(device),
                InputEvent {
                    time: TimeVal { sec: 0, usec: 0 },
                    r#type,
                    code,
                    value
                }
            )),
            "[0.0] /dev/input/event0: EV_KEY BTN_TOUCH DOWN",
            Options {
                get_time: true,
                print_device: true,
            },
            {
                assert_eq!(device, "/dev/input/event0");
                assert_eq!(r#type, input_event_codes::EV_KEY.try_into().unwrap());
                assert_eq!(code, input_event_codes::BTN_TOUCH.try_into().unwrap());
                assert_eq!(value, 1);
            }
        );
    }

    #[test]
    fn test_write_event() {
        macro_rules! run {
            ($event:expr) => {
                let mut device = vec![];
                if let Ok(()) = write_event(&mut device, &$event) {
                    assert_eq!(device, $event.to_ne_bytes());
                } else {
                    assert!(false);
                }
            };
        }
        run!(InputEvent {
            time: TimeVal { sec: 0, usec: 0 },
            r#type: 0,
            code: 0,
            value: 0,
        });
        run!(InputEvent {
            time: TimeVal {
                sec: i64::MAX,
                usec: i64::MAX
            },
            r#type: u16::MAX,
            code: u16::MAX,
            value: i32::MAX,
        });
        run!(InputEvent {
            time: TimeVal { sec: 1, usec: 2 },
            r#type: 3,
            code: 4,
            value: 5,
        });
    }
}
