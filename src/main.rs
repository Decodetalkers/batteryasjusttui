use std::fs;
use std::io::Write;
use std::path::Path;

use dialoguer::theme::ColorfulTheme;
use dialoguer::FuzzySelect;

const ENERGY_PERFORMANCE_PRE: &str =
    "/sys/devices/system/cpu/cpu0/cpufreq/energy_performance_preference";
const ENERGY_PERFORMANCE_AVALABLE: &str =
    "/sys/devices/system/cpu/cpu0/cpufreq/energy_performance_available_preferences";

// ORIGIN
const SCALING_GOVERNOR: &str = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor";
const SCALING_GOVERNOR_AVALABLE: &str =
    "/sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors";

// AMD
pub const AMD_GPU_GOVERNOR: &str = "/sys/class/drm/card0/device/power_dpm_force_performance_level";
const AMD_GPU_GOVERNOR_AVALABLE: [&str; 8] = [
    "auto",
    "low",
    "high",
    "manual",
    "profile_standard",
    "profile_min_sclk",
    "profile_min_mclk",
    "profile_peak",
];

// GLOB
const CPUFREQ: &str = "/sys/devices/system/cpu/cpufreq/*";
const ENERGY: &str = "energy_performance_preference";
const SCALLING: &str = "scaling_governor";

enum ChangeFile {
    Glob { pattern: String, file: String },
    Path { path: String },
}

fn get_selections<P: AsRef<Path>>(
    current_path: P,
    select_path: P,
) -> Option<(String, Vec<String>)> {
    if let (Ok(content), Ok(content_current)) = (
        fs::read_to_string(select_path),
        fs::read_to_string(current_path),
    ) {
        let selects: Vec<String> = content
            .trim()
            .split(' ')
            .map(|unit| unit.to_string())
            .collect();
        Some((content_current, selects))
    } else {
        None
    }
}

fn choose_selection<T: ToString>(
    section: &str,
    current_select: String,
    to_select: &[T],
    save_file: ChangeFile,
) -> i32 {
    let Ok(selection) = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Choose {section}?"))
        .default(0)
        .items(&["Yes", "No"])
        .interact() else {
            return -1;
    };
    if selection == 1 {
        return -1;
    }
    println!("current : {current_select}");
    let Ok(selection) = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Now to choose {section}"))
        .default(0)
        .items(&to_select)
        .interact() else {
            return -1;
    };
    let selected = to_select[selection].to_string();
    match save_file {
        ChangeFile::Glob { pattern, file } => {
            let Ok(paths) = glob::glob(&pattern) else {
                return -1;
            };
            for path in paths.flatten() {
                let Ok(mut file_towrite) = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .truncate(true)
                    .open(path.join(&file)) else {
                    eprintln!("failed to open {path:?}");
                    return -1;
                };
                let Ok(_) = file_towrite.write_all(selected.as_bytes()) else {
                    eprintln!("failed to write {path:?}");
                    return -1;
                };
                let _ = file_towrite.flush();
            }
        }
        ChangeFile::Path { path } => {
            let Ok(mut file) = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .truncate(true)
                .open(path) else {
                return -1;
            };
            let Ok(_) = file.write_all(selected.as_bytes()) else {
                return -1;
            };
            let _ = file.flush();
        }
    }
    selection as i32
}

fn main() {
    let enable_pstate = Path::new(ENERGY_PERFORMANCE_PRE).exists();
    let enable_amd_pstate = Path::new(AMD_GPU_GOVERNOR).exists();

    if enable_pstate {
        println!("You have intel pstate, set scaling to power_save to use performance from pstate");
        println!("then else, please set it to performance to use scaling");
    }

    if enable_amd_pstate {
        println!("You can alse set amd_pstate");
    }

    let Some((scaling_current, scaling_toselect)) = get_selections(SCALING_GOVERNOR, SCALING_GOVERNOR_AVALABLE) else {
        eprintln!("No Scaling Path");
        return;
    };
    println!("current scaling : {scaling_current}");

    let selection = choose_selection(
        "scaling selected",
        scaling_current,
        &scaling_toselect,
        ChangeFile::Glob {
            pattern: CPUFREQ.to_string(),
            file: SCALLING.to_string(),
        },
    );
    let continue_pstate =
        selection != -1 && scaling_toselect[selection as usize] != "performance" && enable_pstate;
    let Some((pstate_current, pstate_toselect)) = get_selections(ENERGY_PERFORMANCE_PRE, ENERGY_PERFORMANCE_AVALABLE) else {
        eprintln!("No pstate Path");
        return;
    };
    if continue_pstate {
        choose_selection(
            "pstate",
            pstate_current,
            &pstate_toselect,
            ChangeFile::Glob {
                pattern: CPUFREQ.to_string(),
                file: ENERGY.to_string(),
            },
        );
    }

    if !enable_amd_pstate {
        return;
    }
    let Ok(amd_current) = fs::read_to_string(AMD_GPU_GOVERNOR) else {
        eprintln!("Cannot read amd pstate file");
        return;
    };
    choose_selection(
        "amd_pstate",
        amd_current,
        &AMD_GPU_GOVERNOR_AVALABLE,
        ChangeFile::Path {
            path: AMD_GPU_GOVERNOR.to_string(),
        },
    );
}
