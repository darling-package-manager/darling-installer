use colored::Colorize as _;
use convert_case::Casing as _;
use std::io::Write as _;

struct Installation {
    modules: Vec<(String, String)>,
}

impl Installation {
    pub fn blank() -> Self {
        Self {
            modules: Vec::new(),
        }
    }
}

struct Module {
    readable_name: &'static str,
    name: &'static str,
    commands: &'static [&'static str],
}

fn has_requirement(command: &str) -> bool {
    if which::which(command).is_ok() {
        println!(
            "\t{} is {}",
            command.cyan().bold(),
            "installed ✔".green().bold()
        );
        true
    } else {
        println!(
            "\t{} is {}",
            command.cyan().bold(),
            "not installed ✘".red().bold()
        );
        false
    }
}

fn main() -> anyhow::Result<()> {
    println!("{} requirements...", "Checking".green().bold());
    let requirements = ["git", "cargo"];
    for requirement in requirements {
        if !has_requirement(requirement) {
            anyhow::bail!(
                "{} {}. Please install it before proceeding.",
                "Missing requirement ".red().bold(),
                requirement.cyan().bold()
            );
        }
    }
    println!(
        "{} All requirements are installed.\n",
        "Good news!".green().bold()
    );

    let mut installation = Installation::blank();

    let name_pattern = regex_macro::regex!(r"(?s)ID=(\S+)");
    let distro_name = name_pattern
        .captures(&std::fs::read_to_string("/etc/os-release")?)
        .map(|cap| cap[1].to_owned());

    if let Some(distro_name) = distro_name {
        print!(
            "It looks like you're on {}. ",
            format!("{distro_name} Linux").cyan().bold()
        );
        std::io::stdout().flush()?;

        let output = String::from_utf8(
            std::process::Command::new("cargo")
                .arg("search")
                .arg(&format!("darling-{distro_name}"))
                .output()?
                .stdout,
        )?;

        if output.is_empty() {
            print!(
                "Currently, {}.\nDo you wish to continue the installation? (Y/n): ",
                format!("there is no locatable darling implementation for {distro_name} Linux's package manager",)
                    .red()
                    .bold()
            );
            std::io::stdout().flush()?;
            if !user_confirmed()? {
                anyhow::bail!("{}", "Cancelling darling installation.".red().bold())
            }
        } else {
            print!(
                "{}.\nDo you wish to install this module? (Y/n): ",
                format!(
                    "There exists an implementation of darling for {distro_name} Linux's package manager."
                ).green().bold()
            );
            std::io::stdout().flush()?;

            if user_confirmed()? {
                installation
                    .modules
                    .push((distro_name.to_case(convert_case::Case::Title), distro_name));
            }
        }

        let modules = vec![Module {
            name: "vscode",
            readable_name: "Visual Studio Code",
            commands: &["code", "codium"],
        }];

        for module in modules {
            if module
                .commands
                .iter()
                .any(|command| which::which(command).is_ok())
            {
                print!("It looks like you have {} installed, which has a supported `darling` module. Install it? (Y/n): ", module.readable_name.cyan().bold());
                std::io::stdout().flush()?;
                if user_confirmed()? {
                    installation
                        .modules
                        .push((module.readable_name.to_owned(), module.name.to_owned()));
                }
            }
        }

        println!();

        println!("{} Darling with modules:", "Installing".green().bold());
        for (proper_name, name) in &installation.modules {
            println!(
                "\t{} {}",
                proper_name.cyan().bold(),
                format!("({})", name).truecolor(150, 150, 150)
            );
        }
        print!("Proceed? (Y/n): ");
        std::io::stdout().flush()?;
        if user_confirmed()? {
            let home = std::env::var("HOME")?;
            let working_directory = home.clone() + "/.tmp/darling";
            std::fs::create_dir_all(working_directory.clone())?;
            std::fs::create_dir_all(home.clone() + "/.local/share/darling")?;
            std::process::Command::new("git")
                .arg("clone")
                .arg("https://github.com/darling-package-manager/darling.git")
                .current_dir(working_directory.clone())
                .spawn()?
                .wait()?;
            std::fs::rename(
                working_directory + "/darling",
                home.clone() + "/.local/share/darling/source",
            )?;
            let mut bashrc = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(home.clone() + "/.bashrc")?;

            writeln!(
                bashrc,
                "\nexport PATH=\"$PATH:{home}/.local/share/darling/source/target/release\""
            )?;

            // Build the source
            std::process::Command::new("cargo")
                .arg("build")
                .arg("--release")
                .current_dir(home.clone() + "/.local/share/darling/source")
                .spawn()?
                .wait()?;

            std::env::set_var(
                "PATH",
                std::env::var("PATH")?
                    + ":"
                    + &home
                    + "/.local/share/darling/source/target/release",
            );

            for (proper_name, name) in &installation.modules {
                print!(
                    "\t{} module for {} {}... ",
                    "Installing".green().bold(),
                    proper_name.cyan().bold(),
                    format!("({})", name).truecolor(150, 150, 150)
                );
                std::io::stdout().flush()?;
                if let Err(error) = std::process::Command::new("darling")
                    .arg("module")
                    .arg("install")
                    .arg(name)
                    .spawn()?
                    .wait()
                {
                    println!("{} {}", "Error:".red().bold(), error);
                }
            }
            println!("{}", "Installation complete!".green().bold());
        }
    }

    Ok(())
}

fn user_confirmed() -> anyhow::Result<bool> {
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer)?;
    Ok(buffer.trim().to_lowercase() == "y")
}
