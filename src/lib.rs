use std::env;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const FRAME_DELAY_MS: u64 = 500;
const FINAL_HOLD_MS: u64 = 1000;
const ANIMATION_STEPS: [usize; 3] = [0, 1, 2];
const FINAL_INDENT: usize = 3;

const MESSAGES: [&str; 6] = [
    "You meant `git`, didn't you? Bold typo.",
    "Cute try. Still looks a lot like `git` was the plan.",
    "That honk was suspiciously close to `git`.",
    "You typed `gut`. Your fingers clearly wanted `git`.",
    "Confidently wrong. Were you aiming for `git`?",
    "Impressive. You missed `git` by one letter.",
];

const GOOSE: &str = r#" _
__(.)<
/___)
 " ""#;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Config {
    animation: bool,
}

#[derive(Debug, Eq, PartialEq)]
enum Command {
    Run,
    ShowConfig,
    SetAnimation(bool),
}

pub fn run_cli<I>(args: I) -> ExitCode
where
    I: IntoIterator<Item = String>,
{
    match parse_command(args) {
        Ok(Command::Run) => {
            run();
            ExitCode::SUCCESS
        }
        Ok(Command::ShowConfig) => {
            let config = load_config().unwrap_or_default();
            print!("{}", format_config(config));
            ExitCode::SUCCESS
        }
        Ok(Command::SetAnimation(enabled)) => match update_animation(enabled) {
            Ok((config, path)) => {
                println!("Saved config to {}", path.display());
                print!("{}", format_config(config));
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("Failed to save config: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

pub fn run() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.subsec_nanos() as usize)
        .unwrap_or(0);

    let message = MESSAGES[now % MESSAGES.len()];
    let config = load_config().unwrap_or_default();

    if config.animation && io::stdout().is_terminal() {
        print_animated_goose(message);
        return;
    }

    println!("{}", render_spoken_frame(FINAL_INDENT, message));
}

fn parse_command<I>(args: I) -> Result<Command, String>
where
    I: IntoIterator<Item = String>,
{
    let collected: Vec<String> = args.into_iter().collect();

    match collected.as_slice() {
        [_bin] => Ok(Command::Run),
        [_bin, config, show] if config == "config" && show == "show" => Ok(Command::ShowConfig),
        [_bin, flag, show] if flag == "--config" && show == "show" => Ok(Command::ShowConfig),
        [_bin, flag, key, value] if flag == "--config" => {
            if key != "animation" {
                return Err(
                    "Unknown config key. Supported keys: animation".to_string(),
                );
            }

            Ok(Command::SetAnimation(parse_bool(value)?))
        }
        [_bin, ..] => Err(
            "Usage: gut [config show] [--config show] [--config animation true|false]"
                .to_string(),
        ),
        [] => Ok(Command::Run),
    }
}

fn parse_bool(value: &str) -> Result<bool, String> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("Invalid boolean `{value}`. Use `true` or `false`.")),
    }
}

fn load_config() -> io::Result<Config> {
    let path = config_path()?;

    match fs::read_to_string(path) {
        Ok(contents) => Ok(parse_config(&contents)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Config::default()),
        Err(error) => Err(error),
    }
}

fn parse_config(contents: &str) -> Config {
    let mut config = Config::default();

    for line in contents.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };

        if key.trim() == "animation" {
            config.animation = matches!(value.trim(), "true");
        }
    }

    config
}

fn update_animation(enabled: bool) -> io::Result<(Config, PathBuf)> {
    let config = Config { animation: enabled };
    let path = config_path()?;
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("Invalid config path"))?;

    fs::create_dir_all(parent)?;
    fs::write(&path, format!("animation={enabled}\n"))?;

    Ok((config, path))
}

fn config_path() -> io::Result<PathBuf> {
    if let Some(base) = env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(base).join("gut").join("config"));
    }

    let home = env::var_os("HOME")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;

    Ok(Path::new(&home).join(".config").join("gut").join("config"))
}

fn format_config(config: Config) -> String {
    format!("animation = {}\n", config.animation)
}

fn print_animated_goose(message: &str) {
    let mut stdout = io::stdout();
    let mut previous_lines = 0;

    for indent in ANIMATION_STEPS {
        let frame = indent_text(GOOSE, indent);
        clear_previous_frame(&mut stdout, previous_lines);
        let _ = writeln!(stdout, "{frame}");
        let _ = stdout.flush();
        previous_lines = count_lines(&frame);
        thread::sleep(Duration::from_millis(FRAME_DELAY_MS));
    }

    let final_frame = indent_text(GOOSE, FINAL_INDENT);
    clear_previous_frame(&mut stdout, previous_lines);
    let _ = writeln!(stdout, "{final_frame}");
    let _ = stdout.flush();
    thread::sleep(Duration::from_millis(FRAME_DELAY_MS));

    clear_previous_frame(&mut stdout, count_lines(&final_frame));
    let spoken_frame = render_spoken_frame(FINAL_INDENT, message);
    let _ = writeln!(stdout, "{spoken_frame}");
    let _ = stdout.flush();
    thread::sleep(Duration::from_millis(FINAL_HOLD_MS));
}

fn clear_previous_frame(stdout: &mut impl Write, line_count: usize) {
    if line_count == 0 {
        return;
    }

    let _ = write!(stdout, "\x1B[{line_count}F\x1B[J");
}

fn count_lines(text: &str) -> usize {
    text.lines().count()
}

fn indent_text(text: &str, indent: usize) -> String {
    let padding = " ".repeat(indent);

    text.lines()
        .map(|line| format!("{padding}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_spoken_frame(indent: usize, message: &str) -> String {
    let goose_lines: Vec<String> = indent_text(GOOSE, indent)
        .lines()
        .map(str::to_string)
        .collect();
    let gap = "  ";

    vec![
        goose_lines[0].clone(),
        format!("{}{}< {}", goose_lines[1], gap, message),
        goose_lines[2].clone(),
        goose_lines[3].clone(),
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::{format_config, indent_text, parse_command, parse_config, render_spoken_frame, Command, Config};

    #[test]
    fn parses_animation_config_command() {
        let command = parse_command([
            "gut".to_string(),
            "--config".to_string(),
            "animation".to_string(),
            "true".to_string(),
        ])
        .unwrap();

        assert_eq!(command, Command::SetAnimation(true));
    }

    #[test]
    fn parses_show_config_command() {
        let command =
            parse_command(["gut".to_string(), "config".to_string(), "show".to_string()]).unwrap();

        assert_eq!(command, Command::ShowConfig);
    }

    #[test]
    fn parses_animation_config_file() {
        let config = parse_config("animation=true\n");

        assert_eq!(config, Config { animation: true });
    }

    #[test]
    fn formats_current_config() {
        let output = format_config(Config { animation: true });

        assert_eq!(output, "animation = true\n");
    }

    #[test]
    fn indents_every_line_of_goose() {
        let indented = indent_text("a\nb", 2);

        assert_eq!(indented, "  a\n  b");
    }

    #[test]
    fn renders_spoken_frame_with_message() {
        let frame = render_spoken_frame(1, "honk");

        assert!(frame.contains("< honk"));
    }
}
