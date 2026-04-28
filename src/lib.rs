use std::env;
use std::fmt;
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

const EN_MESSAGES: [&str; 6] = [
    "You meant `git`, didn't you? Bold typo.",
    "Cute try. Still looks a lot like `git` was the plan.",
    "That honk was suspiciously close to `git`.",
    "You typed `gut`. Your fingers clearly wanted `git`.",
    "Confidently wrong. Were you aiming for `git`?",
    "Impressive. You missed `git` by one letter.",
];

const JA_MESSAGES: [&str; 6] = [
    "`git` のつもりでしたよね。堂々と `gut` ですね。",
    "惜しいです。かなり `git` のつもりだった気配があります。",
    "そのガァガァ、かなり `git` に近いです。",
    "`gut` を打ちましたが、指は `git` を目指していた気がします。",
    "自信はありますが、たぶん `git` ではないです。",
    "`git` を 1 文字で外しました。印象には残ります。",
];

const GOOSE: &str = r#" _
__(.)<
/___)
 " ""#;

const DUCK: &str = r#"       __
  ___( o)>
  \ <_. )
   `---'"#;

const OWL: &str = r#" ,___,
 [O,O]
 /)__)
/--"-""#;

const RANDOM_CHARACTERS: [Character; 3] = [Character::Goose, Character::Duck, Character::Owl];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Language {
    En,
    Ja,
}

impl Default for Language {
    fn default() -> Self {
        Self::En
    }
}

impl Language {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "en" => Ok(Self::En),
            "ja" => Ok(Self::Ja),
            _ => Err(format!("Invalid language `{value}`. Use `en` or `ja`.")),
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::En => write!(f, "en"),
            Self::Ja => write!(f, "ja"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Character {
    Goose,
    Duck,
    Owl,
    Random,
}

impl Default for Character {
    fn default() -> Self {
        Self::Goose
    }
}

impl Character {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "goose" => Ok(Self::Goose),
            "duck" => Ok(Self::Duck),
            "owl" => Ok(Self::Owl),
            "random" => Ok(Self::Random),
            _ => Err(format!(
                "Invalid character `{value}`. Use `goose`, `duck`, `owl`, or `random`."
            )),
        }
    }

    fn art(self) -> &'static str {
        match self {
            Self::Goose => GOOSE,
            Self::Duck => DUCK,
            Self::Owl => OWL,
            Self::Random => unreachable!("random must be resolved before rendering"),
        }
    }

    fn resolve(self, seed: usize) -> Self {
        match self {
            Self::Random => RANDOM_CHARACTERS[seed % RANDOM_CHARACTERS.len()],
            concrete => concrete,
        }
    }
}

impl fmt::Display for Character {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Goose => write!(f, "goose"),
            Self::Duck => write!(f, "duck"),
            Self::Owl => write!(f, "owl"),
            Self::Random => write!(f, "random"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Config {
    animation: bool,
    language: Language,
    character: Character,
}

#[derive(Debug, Eq, PartialEq)]
enum ConfigUpdate {
    Animation(bool),
    Language(Language),
    Character(Character),
}

#[derive(Debug, Eq, PartialEq)]
enum Command {
    Run,
    ShowConfig,
    UpdateConfig(ConfigUpdate),
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
        Ok(Command::UpdateConfig(update)) => match update_config(update) {
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
    let config = load_config().unwrap_or_default();
    let seed = random_seed();
    let message = choose_message(config.language, seed);
    let character = config.character.resolve(seed).art();

    if config.animation && io::stdout().is_terminal() {
        print_animated_character(character, message);
        return;
    }

    println!("{}", render_spoken_frame(FINAL_INDENT, character, message));
}

fn random_seed() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.subsec_nanos() as usize)
        .unwrap_or(0)
}

fn choose_message(language: Language, seed: usize) -> &'static str {
    let pool = match language {
        Language::En => &EN_MESSAGES,
        Language::Ja => &JA_MESSAGES,
    };

    pool[seed % pool.len()]
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
            Ok(Command::UpdateConfig(parse_config_update(key, value)?))
        }
        [_bin, ..] => Err(
            "Usage: gut [config show] [--config show] [--config animation true|false] [--config language en|ja] [--config character goose|duck|owl|random]"
                .to_string(),
        ),
        [] => Ok(Command::Run),
    }
}

fn parse_config_update(key: &str, value: &str) -> Result<ConfigUpdate, String> {
    match key {
        "animation" => Ok(ConfigUpdate::Animation(parse_bool(value)?)),
        "language" => Ok(ConfigUpdate::Language(Language::parse(value)?)),
        "character" => Ok(ConfigUpdate::Character(Character::parse(value)?)),
        _ => Err(
            "Unknown config key. Supported keys: animation, language, character"
                .to_string(),
        ),
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

        let value = value.trim();

        match key.trim() {
            "animation" => config.animation = matches!(value, "true"),
            "language" => {
                if let Ok(language) = Language::parse(value) {
                    config.language = language;
                }
            }
            "character" => {
                if let Ok(character) = Character::parse(value) {
                    config.character = character;
                }
            }
            _ => {}
        }
    }

    config
}

fn update_config(update: ConfigUpdate) -> io::Result<(Config, PathBuf)> {
    let mut config = load_config().unwrap_or_default();

    match update {
        ConfigUpdate::Animation(enabled) => config.animation = enabled,
        ConfigUpdate::Language(language) => config.language = language,
        ConfigUpdate::Character(character) => config.character = character,
    }

    let path = config_path()?;
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("Invalid config path"))?;

    fs::create_dir_all(parent)?;
    fs::write(
        &path,
        format!(
            "animation={}\nlanguage={}\ncharacter={}\n",
            config.animation, config.language, config.character
        ),
    )?;

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
    format!(
        "animation = {}\nlanguage = {}\ncharacter = {}\n",
        config.animation, config.language, config.character
    )
}

fn print_animated_character(character: &str, message: &str) {
    let mut stdout = io::stdout();
    let mut previous_lines = 0;

    for indent in ANIMATION_STEPS {
        let frame = indent_text(character, indent);
        clear_previous_frame(&mut stdout, previous_lines);
        let _ = writeln!(stdout, "{frame}");
        let _ = stdout.flush();
        previous_lines = count_lines(&frame);
        thread::sleep(Duration::from_millis(FRAME_DELAY_MS));
    }

    let final_frame = indent_text(character, FINAL_INDENT);
    clear_previous_frame(&mut stdout, previous_lines);
    let _ = writeln!(stdout, "{final_frame}");
    let _ = stdout.flush();
    thread::sleep(Duration::from_millis(FRAME_DELAY_MS));

    clear_previous_frame(&mut stdout, count_lines(&final_frame));
    let spoken_frame = render_spoken_frame(FINAL_INDENT, character, message);
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

fn render_spoken_frame(indent: usize, character: &str, message: &str) -> String {
    let lines: Vec<String> = indent_text(character, indent)
        .lines()
        .map(str::to_string)
        .collect();
    let gap = "  ";
    let speech_line = if lines.len() > 1 { 1 } else { 0 };

    lines
        .into_iter()
        .enumerate()
        .map(|(index, line)| {
            if index == speech_line {
                format!("{line}{gap}< {message}")
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::{
        format_config, indent_text, parse_command, parse_config, render_spoken_frame, Character,
        Command, Config, ConfigUpdate, Language,
    };

    #[test]
    fn parses_animation_config_command() {
        let command = parse_command([
            "gut".to_string(),
            "--config".to_string(),
            "animation".to_string(),
            "true".to_string(),
        ])
        .unwrap();

        assert_eq!(command, Command::UpdateConfig(ConfigUpdate::Animation(true)));
    }

    #[test]
    fn parses_show_config_command() {
        let command =
            parse_command(["gut".to_string(), "config".to_string(), "show".to_string()]).unwrap();

        assert_eq!(command, Command::ShowConfig);
    }

    #[test]
    fn parses_language_and_character_from_config_file() {
        let config = parse_config("animation=true\nlanguage=ja\ncharacter=random\n");

        assert_eq!(
            config,
            Config {
                animation: true,
                language: Language::Ja,
                character: Character::Random,
            }
        );
    }

    #[test]
    fn formats_current_config() {
        let output = format_config(Config {
            animation: true,
            language: Language::Ja,
            character: Character::Duck,
        });

        assert_eq!(output, "animation = true\nlanguage = ja\ncharacter = duck\n");
    }

    #[test]
    fn indents_every_line_of_character() {
        let indented = indent_text("a\nb", 2);

        assert_eq!(indented, "  a\n  b");
    }

    #[test]
    fn renders_spoken_frame_with_message() {
        let frame = render_spoken_frame(1, "a\nb", "honk");

        assert!(frame.contains("b  < honk"));
    }

    #[test]
    fn resolves_random_character_from_seed() {
        assert_eq!(Character::Random.resolve(0), Character::Goose);
        assert_eq!(Character::Random.resolve(1), Character::Duck);
        assert_eq!(Character::Random.resolve(2), Character::Owl);
        assert_eq!(Character::Random.resolve(3), Character::Goose);
    }
}
