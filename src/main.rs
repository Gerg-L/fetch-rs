use colored::*;
use nix::sys::statvfs::statvfs;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::env;
use std::fmt;
use std::fmt::Debug;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::ops::BitAnd;
use std::path::Path;
use std::result::Result;
use std::str::FromStr;
fn pretty_name() -> Result<String, Error> {
    let mut s = String::new();
    File::open("/etc/os-release")?.read_to_string(&mut s)?;

    let mut pretty_name = String::new();
    for l in s.split('\n') {
        match parse_lines(l.trim().to_string()) {
            Some((key, value)) => match (key.as_ref(), value) {
                ("PRETTY_NAME", val) => {
                    pretty_name = val.clone();
                    Some(val);
                }
                _ => {}
            },
            None => {}
        }
    }

    Ok(pretty_name)
}
fn cat_file(path: &str) -> Result<String, Error> {
    let mut hostname = String::new();
    File::open(path)?
        .read_to_string(&mut hostname)
        .expect("Could not read file {path}");
    Ok(hostname.trim().to_string())
}

fn parse_lines(l: String) -> Option<(String, String)> {
    let words: Vec<&str> = l.splitn(2, '=').collect();
    if words.len() < 2 {
        return None;
    }
    let mut trim_value = String::from(words[1]);

    if trim_value.starts_with('"') {
        trim_value.remove(0);
    }
    if trim_value.ends_with('"') {
        let len = trim_value.len();
        trim_value.remove(len - 1);
    }

    return Some((String::from(words[0]), trim_value));
}

pub fn get_uptime() -> isize {
    let data = cat_file("/proc/uptime").unwrap();
    let numbers: Vec<&str> = data.split(' ').collect();
    let uptime: Vec<&str> = numbers[0].split('.').collect();
    FromStr::from_str(uptime[0]).unwrap()
}

fn format_dhms<T: TryInto<usize> + TryFrom<usize> + BitAnd<Output = T>>(seconds: T) -> String
where
    <T as TryFrom<usize>>::Error: Debug,
    <T as TryInto<usize>>::Error: Debug,
{
    const MINUTE: usize = 60;
    const HOUR: usize = 3_600;
    const DAY: usize = 86_400;

    let seconds: usize = if std::mem::size_of::<T>() <= std::mem::size_of::<usize>() {
        seconds.try_into().unwrap()
    } else {
        (seconds & usize::MAX.try_into().unwrap())
            .try_into()
            .unwrap()
    };
    let mut compound_duration = String::new();
    if seconds == 0 {
        compound_duration.push_str("0s ");
        return compound_duration;
    }

    let mut sec = seconds % DAY;
    let ds = seconds / DAY;
    // days
    if ds != 0 {
        compound_duration.push_str(format!("{}d ", ds).as_str());
    }

    // hours
    let hs = sec / HOUR;
    sec %= HOUR;
    if hs != 0 {
        compound_duration.push_str(format!("{}h ", hs).as_str());
    }

    // minutes
    let ms = sec / MINUTE;
    sec %= MINUTE;
    if ms != 0 {
        compound_duration.push_str(format!("{}m ", ms).as_str());
    }

    // seconds
    if sec != 0 {
        compound_duration.push_str(format!("{}s ", sec).as_str());
    }

    compound_duration
}
fn strip_path(path: String) -> String {
    let file_name = Path::new(&path).file_name();
    return file_name.unwrap().to_string_lossy().to_string();
}
#[derive(Debug)]
pub enum Error {
    UnsupportedSystem,
    ExecFailed(io::Error),
    IO(io::Error),
    SystemTime(std::time::SystemTimeError),
    General(String),
    Unknown,
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;
        match *self {
            UnsupportedSystem => write!(fmt, "System is not supported"),
            ExecFailed(ref e) => write!(fmt, "Execution failed: {}", e),
            IO(ref e) => write!(fmt, "IO error: {}", e),
            SystemTime(ref e) => write!(fmt, "System time error: {}", e),
            General(ref e) => write!(fmt, "Error: {}", e),
            Unknown => write!(fmt, "An unknown error occurred"),
        }
    }
}
impl std::error::Error for Error {
    fn description(&self) -> &str {
        use self::Error::*;
        match *self {
            UnsupportedSystem => "unsupported system",
            ExecFailed(_) => "execution failed",
            IO(_) => "io error",
            SystemTime(_) => "system time",
            General(_) => "general error",
            Unknown => "unknown error",
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        use self::Error::*;
        match *self {
            UnsupportedSystem => None,
            ExecFailed(ref e) => Some(e),
            IO(ref e) => Some(e),
            SystemTime(ref e) => Some(e),
            General(_) => None,
            Unknown => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IO(e)
    }
}

impl From<std::time::SystemTimeError> for Error {
    fn from(e: std::time::SystemTimeError) -> Error {
        Error::SystemTime(e)
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(e: Box<dyn std::error::Error>) -> Error {
        Error::General(e.to_string())
    }
}

struct MemInfo {
    pub total: u64,
    pub used: u64,
}

fn mem_info() -> Result<MemInfo, Error> {
    {
        let mut s = String::new();
        File::open("/proc/meminfo")?.read_to_string(&mut s)?;
        let mut meminfo_hashmap = HashMap::new();
        for line in s.lines() {
            let mut split_line = line.split_whitespace();
            let label = split_line.next();
            let value = split_line.next();
            if value.is_some() && label.is_some() {
                let label = label.unwrap().split(':').nth(0).ok_or(Error::Unknown)?;
                let value = value.unwrap().parse::<u64>().ok().ok_or(Error::Unknown)?;
                meminfo_hashmap.insert(label, value);
            }
        }
        let total = *meminfo_hashmap.get("MemTotal").ok_or(Error::Unknown)?;
        let free = *meminfo_hashmap.get("MemFree").ok_or(Error::Unknown)?;
        let buffers = *meminfo_hashmap.get("Buffers").ok_or(Error::Unknown)?;
        let cached = *meminfo_hashmap.get("Cached").ok_or(Error::Unknown)?;
        let slab = *meminfo_hashmap.get("Slab").ok_or(Error::Unknown)?;
        let used = total - free - buffers - cached - slab;
        Ok(MemInfo {
            total: total * 1000,
            used: used * 1000,
        })
    }
}

pub fn iec(n: u64) -> String {
    let units = ["", "k", "M", "G", "T", "P", "E", "Z", "Y"];

    let i = (n as f64).log(1000_f64).floor() as u32;
    let p = 1000_u64.pow(i);
    let s = (n as f64) / (p as f64);

    format!("{:.1}{}", s, units[i as usize])
}

fn get_file_system_info() -> Vec<Vec<String>> {
    const FS_SPEC: usize = 0;
    const FS_FILE: usize = 1;
    let file = File::open("/proc/mounts").expect("Error opening /proc/mounts");
    let reader = BufReader::new(&file);
    let mut r_vec = Vec::new();
    let mut vs_ids = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                let fields: Vec<&str> = line.split_whitespace().collect();
                if !fields[FS_SPEC].contains("/") {
                    continue;
                }
                let statvfs = match statvfs(fields[FS_FILE]) {
                    Ok(s) => s,
                    Err(err) => {
                        println!("Error: {}", err);
                        continue;
                    }
                };
                if vs_ids.contains(&statvfs.filesystem_id()) {
                    continue;
                } else {
                    vs_ids.push(statvfs.filesystem_id());
                }
                let size = statvfs.blocks() * statvfs.block_size();
                let avail = statvfs.blocks_free() * statvfs.block_size();
                if size == 0 {
                    continue;
                }
                let r_string = vec![fields[FS_FILE].to_string(), iec(size), iec(size - avail)];

                r_vec.push(r_string);
            }
            Err(err) => println!("Error: {}", err),
        }
    }
    return r_vec;
}

fn main() {
    let user = env::var("USER").expect("$USER not set").white();
    let hostname = cat_file("/etc/hostname")
        .expect("Could not read /etc/hostname")
        .red();
    let pretty_name = pretty_name()
        .expect("Could Not access /etc/os-release")
        .yellow();
    let kernel = cat_file("/proc/sys/kernel/osrelease")
        .expect("Could not read /proc/sys/kernel/osrelease")
        .green();
    let uptime_formatted = format_dhms(get_uptime()).blue();
    let shell = strip_path(env::var("SHELL").expect("$SHELL not set")).magenta();
    let editor = strip_path(env::var("EDITOR").expect("$EDITOR not set")).bright_black();

    let memory = mem_info().expect("Could not get and parse memory info from /proc/meminfo");
    let mem_used = iec(memory.used);
    let mem_total = iec(memory.total);

    let dot = " ";
    let dots = format!(
        "{}{}{}{}{}{}{}{}",
        dot.white(),
        dot.red(),
        dot.yellow(),
        dot.green(),
        dot.cyan(),
        dot.magenta(),
        dot.black(),
        dot.blue()
    );

    let s = vec![
        "".white(),
        "".red(),
        "".yellow(),
        "".green(),
        "".cyan(),
        "".magenta(),
        "".bright_black(),
        "".blue(),
        "".white(),
        "".bright_blue(),
    ];
    let nixos = format!(
        "   _  ___      ____  ____
  / |/ (_)_ __/ __ \\/ __/
 /    / /\\ \\ / /_/ /\\ \\
/_/|_/_//_\\_\\\\____/___/"
    )
    .cyan();
    let output = format!(
        "{nixos}
  ╭───────────╮
  │ {}  user   │ {user}
  │ {}  hname  │ {hostname}
  │ {}  distro │ {pretty_name}
  │ {}  kernel │ {kernel}
  │ {}  uptime │ {uptime_formatted}
  │ {}  shell  │ {shell}
  │ {}  editor │ {editor}
  │ {}  memory │ {mem_total}/{mem_used}
  ├───────────┤
  │ {}  colors │ {dots}
  ╰───────────╯
",
        s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7], s[8]
    );
    print!("{}", output);
    let filesys = get_file_system_info();
    if filesys.len() == 0 {
        return;
    }
    let mut drives = Vec::new();
    fn line_str(v: &str) -> Vec<String> {
        let mut lines = Vec::new();
        for _ in 0..v.len() {
            lines.push("─".to_string());
        }
        return lines;
    }
    for vec in filesys.as_slice() {
        drives.push("  ╭".to_string());
        drives.append(&mut line_str(&vec[0]));
        drives.push("──".to_string());
        drives.push("┬".to_string());
        drives.append(&mut line_str(&vec[1]));
        drives.push("─".to_string());
        drives.append(&mut line_str(&vec[2]));
        drives.push("╮\n".to_string());
        drives.push(format!(
            "  │ {}|{}/{}│\n",
            vec[0].bright_black(),
            vec[1].cyan(),
            vec[2].white()
        ));
        drives.push("  ╰".to_string());
        drives.append(&mut line_str(&vec[0]));
        drives.push("──".to_string());
        drives.push("┴".to_string());
        drives.append(&mut line_str(&vec[1]));
        drives.push("─".to_string());
        drives.append(&mut line_str(&vec[2]));
        drives.push("╯\n".to_string());
    }
    let drives_lined = drives.join("");
    print!("{}", drives_lined);
}
