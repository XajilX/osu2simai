use std::{error::Error, fs::File, io::{BufRead, BufReader, Write}, path::PathBuf};

use clap::Parser;

const TAG_TM: &'static str = "[TimingPoints]";
const TAG_HO: &'static str = "[HitObjects]";

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// Osu beatmap need to convert
    file: PathBuf,

    /// Key config
    #[arg(short, value_name="config")]
    key: Option<String>
}

#[allow(unused)]
#[derive(Debug)]
enum Timing {
    Red {
        start: i32,
        bpm: f64,
        pat_num: i32
    },
    Green {
        start: i32,
        speed: f64
    }
}

impl Timing {
    pub fn parse(str_tm: &str) -> Self {
        let params = str_tm.trim().split(',')
            .map(|str| {
                str.parse::<f64>().ok()
            }).collect::<Vec<_>>();
        if params[6].unwrap() == 0. {
            Self::Green {
                start: params[0].unwrap() as i32,
                speed: 100. / (-params[1].unwrap())
            }
        } else {
            Self::Red {
                start: params[0].unwrap() as i32,
                bpm: 60000. / params[1].unwrap(),
                pat_num: params[3].unwrap() as i32,
            }
        }
        
    }

    pub fn start(&self) -> i32 {
        match self {
            Self::Red { start,.. } => *start,
            Self::Green { start,.. } => *start,
        }
    }

    pub fn bpm(&self) -> f64 {
        match self {
            Self::Red { start: _, bpm,.. } => *bpm,
            Self::Green { .. } => panic!(),
        }
    }

}

#[derive(Debug, Clone)]
enum HitObj {
    Tap {
        pos: i32,
        start: i32,
    },
    Long {
        pos: i32,
        start: i32,
        end: i32
    }
}

impl HitObj {
    pub fn parse(str_obj: &str) -> Self {
        let params = str_obj.split_once(':').unwrap().0.split(',')
            .map(|str| {
                str.parse::<i32>().ok()
            }).collect::<Vec<_>>();
        let (x, start) = (params[0].unwrap(), params[2].unwrap());
        if params[3].unwrap() & 1 == 1 {
            Self::Tap {
                pos: x * 4 / 512,
                start
            }
        } else {
            Self::Long {
                pos: x * 4 / 512,
                start,
                end: params[5].unwrap()
            }
        }
    }

    pub fn pos(&self) -> i32 {
        match self {
            Self::Tap { pos, .. } => *pos,
            Self::Long { pos, .. } => *pos
        }
    }

    pub fn start(&self) -> i32 {
        match self {
            Self::Tap { pos: _, start } => *start,
            Self::Long { pos: _, start, .. } => *start
        }
    }
}

fn gcd(x: i32, y: i32) -> i32 {
    if y == 0 { x } else { gcd(y, x % y) }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    let notepos = match args.key {
        None => { Vec::from([1, 2, 3, 4, 5, 6, 7, 8]) },
        Some(str) => {
            let mut ret = Vec::new();
            for c in str.chars() {
                if c.is_ascii_digit() && c <= '8' && c >= '1' {
                    ret.push(c.to_digit(10).unwrap())
                } else {
                    return Err("Not a valid key config".into());
                }
            }
            ret
        }
    };

    let osu_file = File::open(args.file)?;
    let (mut is_tm, mut is_ho) = (false, false);

    let mut timings: Vec<Timing> = Vec::new();
    let mut hitobjs: Vec<HitObj> = Vec::new();

    for line in BufReader::new(osu_file).lines()
            .map(|x| { x.unwrap() }) {
        if line.contains(TAG_TM) {
            is_tm = true;
            is_ho = false;
            continue;
        }
        if line.contains(TAG_HO) {
            is_ho = true;
            is_tm = false;
            continue;
        }
        if line.chars().next() == Some('[') {
            is_ho = false;
            is_tm = false;
            continue;
        }

        if line.trim().len() == 0 {
            continue;
        }
        if is_tm {
            timings.push(Timing::parse(&line));
        } else if is_ho {
            hitobjs.push(HitObj::parse(&line));
        }
    }

    let mut sim_file = File::create("maidata.txt")?;
    let mut bpm = timings[0].bpm();
    let mut time = hitobjs[0].start();
    let mut pat_div = 0;
    writeln!(&mut sim_file, "&first={}", time as f64 / 1000.)?;
    write!(&mut sim_file, "({:.3})", bpm)?;
    let mut iobj = hitobjs.iter().peekable();
    let mut itm = timings.iter().skip(1).peekable();
    while let Some(_) = iobj.peek() {
        let mut time_next = std::i32::MAX;

        while let Some(timing) = itm.peek() {
            if time == timing.start() {
                match timing {
                    &&Timing::Red { start: _, bpm: bpm_t,.. } => {
                        bpm = bpm_t;
                        writeln!(&mut sim_file, "")?;
                        write!(&mut sim_file, "({:.3})", bpm)?;
                    },
                    &&Timing::Green { start: _, speed: _ } => {}
                }
                itm.next();
            } else {
                time_next = timing.start();
                break;
            }
        }

        let mut hits: Vec<HitObj> = Vec::new();
        while let Some(obj) = iobj.peek() {
            if time == obj.start() {
                hits.push((*obj).clone());
                iobj.next();
            } else {
                time_next = time_next.min(obj.start());
                break;
            }
        }
        
        #[allow(unused_assignments)]
        let (mut pat_n, mut pat_d) = (1, 0);
        if time_next != std::i32::MAX {
            let pat = ((time_next - time) as f64 * 96. / 60000. * bpm).round() as i32;
            let g = gcd(pat, 384);
            (pat_n, pat_d) = (pat / g, 384 / g);
            if pat_div == 0 || pat_div % pat_d != 0 || (pat_div % pat_d == 0 && pat_div / pat_d >= 8) {
                writeln!(&mut sim_file, "")?;
                write!(&mut sim_file, "{{{pat_d}}}")?;
                pat_div = pat_d;
            } else {
                let x = pat_div / pat_d;
                if x != 1 {
                    pat_n *= x;
                }
            }
        }
        for (i, obj) in hits.iter().enumerate() {
            if i > 0 {
                write!(&mut sim_file, "/")?;
            }
            write!(&mut sim_file, "{}", notepos[obj.pos() as usize])?;
            match obj {
                &HitObj::Long { pos: _, start, end } => {
                    let pat = ((end - start) as f64 * 96. / 60000. * bpm).round() as i32;
                    let g = gcd(pat, 384);
                    let (len_n, len_d) = (pat / g, 384 / g);
                    write!(&mut sim_file, "h[{len_d}:{len_n}]")?;
                },
                _ => {}
            }
        }
        write!(&mut sim_file, "{}", ",".repeat(pat_n as usize))?;
        time = time_next;
    }
    Ok(())
}
