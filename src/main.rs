use clap::Parser;
use cli_clipboard::{ClipboardContext, ClipboardProvider};
use rand::{thread_rng, Rng};
use std::env::{var, VarError};
use std::io::prelude::*;
use std::ops::Range;
use std::{fs, fs::File, path::Path};

struct NidConfig {
    save_path: String,
    failed: bool,
}

impl NidConfig {
    fn failed() -> NidConfig {
        NidConfig {
            save_path: String::new(),
            failed: true,
        }
    }

    fn success(path: String) -> NidConfig {
        NidConfig {
            save_path: path,
            failed: false,
        }
    }
}

#[derive(Parser, Debug)]
struct Cli {
    /// Wether or not to save the id, to avoid generating duplicate. Saved ids file destination is
    /// manage by the config file (~/.nid/nid_config)
    #[arg(short, long, default_value_t = false)]
    save: bool,

    /// List the saved ids already used
    #[arg(short, long)]
    list: bool,

    /// Depending on commands, shows human readable input
    #[arg(long)]
    verbose: bool,

    /// Whether to save it to clipboard or not
    #[arg(short, long)]
    clip: bool,
}

const BASE_DIR_PATH: &str = "~/.nid";
const BASE_SAVED_FILE_PATH: &str = "~/.nid/nid_saved";
const BASE_CONFIG_FILE_PATH: &str = "~/.nid/nid_config";

fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    let config_file_exists = check_env_dir();

    if !config_file_exists {
        initialize_base_dir()?;
    }

    let config = match read_config() {
        Ok(c) => c,
        Err(_) => NidConfig::failed(),
    };

    if args.list {
        let saved_ids = read_saved_ids(&config);
        print_saved_ids(saved_ids, args.verbose);
        return Ok(());
    }

    // Features start here
    let mut random: String;

    if args.save && !config.failed {
        let resolved_path = match resolve_path(&config.save_path) {
            Ok(v) => v,
            Err(_) => panic!("Error there"),
        };
        let file_content = fs::read_to_string(Path::new(resolved_path.as_str()));
        let file_content = match file_content {
            Ok(x) => x,
            Err(_) => String::from(""),
        };

        let mut file_content = file_content.trim_end().split("\n").collect::<Vec<&str>>();

        random = generate_random_id();

        while file_content.contains(&random.as_str()) {
            random = generate_random_id();
        }

        file_content.push(&random);
        let file_content = file_content.join("\n");

        let success = fs::write(Path::new(resolved_path.as_str()), &file_content);

        match success {
            Err(e) => println!("Error saving: {:?}", e),
            _ => (),
        }
    } else {
        random = generate_random_id();
    }

    if args.verbose {
        println!("New id : {}", random);
    } else {
        println!("{}", random);
    }

    if args.clip {
        let clipboard_ctx = ClipboardContext::new();

        if let Ok(mut ctx) = clipboard_ctx {
            match ctx.set_contents(String::from(random)) {
                Ok(()) => println!("Saved to clipboard!"),
                Err(_) => println!("Error: could not save to clipboard"),
            }
        } else {
            println!("Error: could not save to clipboard");
        }
    }

    if config.failed {
        println!("The config is broken, therefore no id has been saved");
    }

    Ok(())
}

fn print_saved_ids(saved_ids: Vec<String>, verbose: bool) {
    if verbose {
        println!("List of used ids:");
    }

    for i in saved_ids {
        if verbose {
            println!("\t- {}", i);
        } else {
            println!("{}", i);
        }
    }
}

fn read_saved_ids(config: &NidConfig) -> Vec<String> {
    let saved_id_content = read_file_content(config.save_path.clone());
    let ids = saved_id_content.trim().split("\n").collect::<Vec<&str>>();
    let mut ids_array: Vec<String> = vec![];

    for i in ids {
        ids_array.push(i.to_string());
    }

    ids_array
}

fn resolve_path(path: &str) -> Result<String, VarError> {
    let home_dir = var("HOME")?;

    if path.starts_with("~") {
        let p = &path[1..];
        let h = home_dir.as_str();
        let formatted_path = format!("{}{}", h, p);
        return Ok(formatted_path);
    }

    Ok(String::from(path))
}

fn read_file_content(path: String) -> String {
    let resolved_path = match resolve_path(&path) {
        Ok(v) => v,
        Err(_) => panic!("Error there"),
    };
    let file_content = fs::read_to_string(Path::new(resolved_path.as_str()));
    let file_content = match file_content {
        Ok(x) => x,
        Err(_) => String::from(""),
    };
    file_content
}

fn read_config() -> Result<NidConfig, VarError> {
    let resolved_path = resolve_path(BASE_CONFIG_FILE_PATH)?;
    let nid_config_path = Path::new(resolved_path.as_str());
    let nid_config_file = match fs::read_to_string(nid_config_path) {
        Ok(file_content) => file_content,
        Err(_) => String::from(""),
    };

    let config = nid_config_file.split("\n").collect::<Vec<&str>>();
    let mut save_path = "";

    for i in config {
        let c: Vec<&str> = i.split("=").collect();
        let trimed = c[0].trim();

        if trimed == "save_path" {
            save_path = c[1].trim();
        }
    }

    let config = NidConfig::success(String::from(save_path));

    Ok(config)
}

fn initialize_base_dir() -> std::io::Result<bool> {
    let resolved_path = match resolve_path(BASE_DIR_PATH) {
        Ok(v) => v,
        Err(_) => panic!("Error there"),
    };
    let path = Path::new(resolved_path.as_str());
    let success_dir = match fs::create_dir(path) {
        Ok(_) => true,
        Err(e) => {
            println!("{:?}", e);
            false
        }
    };

    if !success_dir {
        return Ok(success_dir);
    }

    let resolved_path = match resolve_path(BASE_CONFIG_FILE_PATH) {
        Ok(v) => v,
        Err(_) => panic!("Error there"),
    };
    let config_path = Path::new(resolved_path.as_str());
    let mut config_file = File::create(config_path)?;

    config_file.write_all(b"save_path = ~/.nid/nid_saved")?;

    let resolved_path = match resolve_path(BASE_SAVED_FILE_PATH) {
        Ok(v) => v,
        Err(_) => panic!("Error there"),
    };
    let saved_path = Path::new(resolved_path.as_str());
    File::create(saved_path)?;

    Ok(true)
}

fn check_env_dir() -> bool {
    let resolved_path = match resolve_path(BASE_DIR_PATH) {
        Ok(v) => v,
        Err(_) => panic!("Error there"),
    };
    let path = Path::new(resolved_path.as_str());
    let exists = path.exists();
    let is_dir = path.is_dir();

    exists && is_dir
}

fn generate_range(range: Range<u8>) -> u8 {
    let rng: u8 = thread_rng().gen_range(range);
    rng
}

fn generate_range_chars() -> u8 {
    let rng: u8 = thread_rng().gen_range(65..91);
    let rng2: u8 = thread_rng().gen_range(97..123);
    let frng: u8 = thread_rng().gen_range(0..=1);
    if frng == 0 {
        rng
    } else {
        rng2
    }
}

fn generate_random_id() -> String {
    let mut id_str: [char; 3] = [' ', ' ', ' '];

    let mut x = 0;

    loop {
        if x == id_str.len() {
            break;
        }

        let random_str = generate_range_chars();
        id_str[x] = random_str as char;

        x += 1;
    }

    let random_number = generate_range(10..100);
    let id = format!("{}{}{}{}", id_str[0], id_str[1], id_str[2], random_number);

    id
}
