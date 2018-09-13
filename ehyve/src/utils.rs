#[cfg(target_os = "linux")]
use std::fs::File;
#[cfg(target_os = "linux")]
use std::io::Read;
use raw_cpuid::CpuId;

#[cfg(target_os = "macos")]
use macos::error::*;
#[cfg(target_os = "linux")]
use linux::error::*;
#[cfg(target_os = "windows")]
use windows::error::*;
#[cfg(target_os = "windows")]
use kernel32;

pub fn parse_mem(mem: &str) -> Result<usize> {
    let (num, postfix): (String, String) = mem.chars().partition(|&x| x.is_numeric());
    let num = num.parse::<usize>().map_err(|_| Error::ParseMemory)?;

    let factor = match postfix.as_str() {
        "E" | "e" => 1 << 60 as usize,
        "P" | "p" => 1 << 50 as usize,
        "T" | "t" => 1 << 40 as usize,
        "G" | "g" => 1 << 30 as usize,
        "M" | "m" => 1 << 20 as usize,
        "K" | "k" => 1 << 10 as usize,
        _ => return Err(Error::ParseMemory)
    };

    Ok(num*factor)
}

/// Returns the CPU frequency
pub fn cpufreq() -> Result<u32> {
    let cpuid = CpuId::new();

    if let Some(freq) = cpuid.get_processor_frequency_info() {
        return Ok(freq.processor_base_frequency() as u32);
    }

    #[cfg(target_os = "windows")]
	{
        let mut freq: i64 = 0;

        unsafe {
            if kernel32::QueryPerformanceFrequency(&mut freq) != 0 {
                return Ok((freq / 1000000) as u32);
            }
        }
    }

	#[cfg(target_os = "linux")]
	{
		let mut content = String::new();

    	// If the file cpuinfo_max_freq exists, parse the content and return the frequency
    	if let Ok(mut file) = File::open("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq") {
        	file.read_to_string(&mut content).map_err(|_| Error::MissingFrequency)?;
        	return content.trim().parse::<u32>().map_err(|_| Error::MissingFrequency).map(|x| x / 1000);
    	}
    	// otherwise use the more acurate cpuinfo file and search for the right line
    	else if let Ok(mut file) = File::open("/proc/cpuinfo") {
        	file.read_to_string(&mut content).expect("Couldnt read!");

        	for line in content.lines() {
            	if line.starts_with("cpu MHz") {
                	return line.split(':').skip(1).next().ok_or(Error::MissingFrequency)?
                    	.trim().parse::<f32>().map_err(|_| Error::MissingFrequency).map(|x| x as u32);
            	}
        	}
    	}
	}

    // ups shouldn't happened ..
    Err(Error::MissingFrequency)
}
