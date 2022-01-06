use duct::cmd;
use std::{
    error::Error,
    fmt::Display,
    fs::{self, File},
    io::{self, BufReader, Read, Stdout, Write},
    path::{Path, PathBuf},
    process::{self, Command, Stdio},
    result,
    time::Instant,
};

#[derive(Default)]
struct KyouproResult {
    entry_name: String,
    ac: usize,
    wa: usize,
    average_us: u128,
}

impl KyouproResult {
    pub fn set_average_us(&mut self, average_us: u128) {
        self.average_us = average_us;
    }

    pub fn set_entry_name(&mut self, name: &str) {
        self.entry_name = name.to_string();
    }

    pub fn count_ac(&mut self) {
        self.ac += 1;
    }

    pub fn count_wa(&mut self) {
        self.wa += 1;
    }
}

impl Display for KyouproResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:<13}\tAC×{}\tWA×{}\t{:>7}us",
            self.entry_name, self.ac, self.wa, self.average_us
        )
    }
}

fn exe_measurement(
    exe: &Path,
    io_files: &Vec<(PathBuf, String)>,
    try_times: usize,
) -> KyouproResult {
    println!("ケース名\t結果\t実行時間\t");
    let mut invoke_time = Vec::new();
    let mut kyoupro_result = KyouproResult::default();
    kyoupro_result.set_entry_name(exe.file_name().unwrap().to_str().unwrap());
    for (in_file, out_txt) in io_files {
        let mut output = String::new();

        let mut elapsed_vec = Vec::new();
        for _ in 0..try_times {
            let file = File::open(in_file).unwrap();
            let start = Instant::now();
            output = cmd!(exe).stdin_file(file).read().unwrap();
            let end = start.elapsed();
            elapsed_vec.push(end.as_micros());
        }
        let elapsed_ave = elapsed_vec.iter().sum::<u128>() / elapsed_vec.len() as u128;
        invoke_time.push(elapsed_ave);
        let result = if &output == out_txt {
            kyoupro_result.count_ac();
            "AC"
        } else {
            kyoupro_result.count_wa();
            "WA"
        };
        // let result = output;

        println!(
            "{}\t{}\t{}us\t",
            in_file.file_name().unwrap().to_str().unwrap(),
            result,
            invoke_time.last().unwrap(),
        );
    }

    kyoupro_result.set_average_us(invoke_time.iter().sum::<u128>() / invoke_time.len() as u128);
    kyoupro_result
}

fn search_file_with_extension(
    search_dir: &Path,
    extension: &str,
) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(search_dir)? {
        let item = entry?;
        if item.file_type()?.is_file() && item.path().extension().unwrap_or_default().eq(extension)
        {
            files.push(item.path());
        }
    }

    Ok(files)
}

fn main() -> std::io::Result<()> {
    let txt_extension = "txt";
    let exe_extension = "exe";
    let try_times = 1;

    let mut io_files = search_file_with_extension(Path::new("./in"), txt_extension)
        .unwrap()
        .into_iter()
        .map(|f| {
            let out_file = PathBuf::from(format!(
                "./out/{}",
                f.file_name().unwrap().to_str().unwrap()
            ));
            let out_file = File::open(out_file).unwrap();
            let mut reader = BufReader::new(out_file);
            let mut buf = String::new();
            reader.read_to_string(&mut buf).unwrap();

            (f, buf.trim().to_string())
        })
        .collect::<Vec<(PathBuf, String)>>();
    io_files.sort_by_key(|f| f.clone());
    let exe_files = search_file_with_extension(Path::new("./exe"), exe_extension).unwrap();

    let mut result_vec = Vec::new();

    for exe in exe_files {
        println!("{}", exe.file_name().unwrap().to_str().unwrap());
        let average = exe_measurement(&exe, &io_files, try_times);
        result_vec.push(average);
    }
    println!();

    result_vec.sort_by_key(|f| f.average_us);

    println!("\tEntry Name\tAC\tWA\t実行時間");
    for (rank, result) in result_vec.iter().enumerate() {
        println!("{}.\t{}", rank + 1, result);
    }

    Ok(())
}
