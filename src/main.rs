use colored::Colorize as _;
use convert_case::Casing as _;
use std::io::Write as _;

struct Installation {
    modules: Vec<(String, String)>,
    is_reinstallation: bool,
}

impl Installation {
    pub fn blank() -> Self {
        Self {
            modules: Vec::new(),
            is_reinstallation: false,
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
        println!("\t{} is {}", command.cyan().bold(), "installed ✔".green().bold());
        true
    } else {
        println!("\t{} is {}", command.cyan().bold(), "not installed ✘".red().bold());
        false
    }
}

fn main() -> anyhow::Result<()> {
    let mut installation = Installation::blank();

    // Check if darling is already installed
    if which::which("darling").is_ok() {
        print!(
            "It looks like {} is already {}. Would you like to {}? (Y/n): ",
            "darling".cyan().bold(),
            "installed".green().bold(),
            "reinstall it".yellow().bold()
        );
        std::io::stdout().flush()?;

        if !user_confirmed()? {
            anyhow::bail!("{}", "Cancelling installation.".red().bold());
        }

        installation.is_reinstallation = true;
    }

    // Check requirements
    println!("\n{} requirements...", "Checking".green().bold());
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
    println!("{} All requirements are installed.\n", "Good news!".green().bold());

    // Get OS information
    let name_pattern = regex_macro::regex!(r"(?s)ID=(\S+)");
    let distro_name = name_pattern
        .captures(&std::fs::read_to_string("/etc/os-release")?)
        .map(|cap| cap[1].to_owned());
    if let Some(distro_name) = distro_name {
        print!("It looks like you're on {}. ", format!("{distro_name} Linux").cyan().bold());
        std::io::stdout().flush()?;

        // Check for darling implementation for distro
        let output = String::from_utf8(
            std::process::Command::new("cargo")
                .arg("search")
                .arg(&format!("darling-{distro_name}"))
                .output()?
                .stdout,
        )?;

        // No implementation!
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
        }
        // Implementation found!
        else {
            print!(
                "{}.\nDo you wish to install this module? (Y/n): ",
                format!("There exists an implementation of darling for {distro_name} Linux's package manager")
                    .green()
                    .bold()
            );
            std::io::stdout().flush()?;

            if user_confirmed()? {
                installation
                    .modules
                    .push((distro_name.to_case(convert_case::Case::Title), distro_name));
            }
        }

        // Find applicable modules
        let modules = vec![
            Module {
                name: "cargo",
                readable_name: "Cargo",
                commands: &["cargo"],
            },
            Module {
                name: "vscode",
                readable_name: "Visual Studio Code",
                commands: &["code", "codium"],
            },
        ];
        println!("\n{} for applicable modules...", "Scanning".green().bold());
        let mut applicable_modules = Vec::new();
        for module in modules {
            if module.commands.iter().any(|command| which::which(command).is_ok()) {
                applicable_modules.push(module);
            }
        }
        println!(
            "Based on applications you have installed, there {} {} module{} you may find useful.",
            if applicable_modules.len() == 1 { "is" } else { "are" },
            applicable_modules.len().to_string().cyan().bold(),
            if applicable_modules.len() == 1 { "s" } else { "" }
        );

        // Get user selected moduels to install
        println!("Please select the modules you'd like to install (you can change this at any time):");
        let choices = dialoguer::MultiSelect::new()
            .items(
                &applicable_modules
                    .iter()
                    .map(|module| {
                        format!(
                            "{} {}",
                            module.readable_name.cyan().bold(),
                            format!("({})", module.name).truecolor(150, 150, 150)
                        )
                    })
                    .collect::<Vec<_>>(),
            )
            .interact()?;
        for choice in choices {
            let display = &applicable_modules[choice];
            installation
                .modules
                .push((display.readable_name.to_owned(), display.name.to_owned()));
        }

        // Print installation info
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

        // Proceed with the installation
        if user_confirmed()? {
            // Reinstallation - remove old darling directory
            let home = std::env::var("HOME")?;
            if installation.is_reinstallation {
                _ = std::fs::remove_dir_all(home.clone() + "/.local/share/darling");
            }

            // Create locations
            let working_directory = home.clone() + "/.tmp/darling";
            std::fs::create_dir_all(working_directory.clone())?;
            std::fs::create_dir_all(home.clone() + "/.local/share/darling")?;

            // Download the source to the correct location
            std::process::Command::new("git")
                .arg("clone")
                .arg("https://github.com/darling-package-manager/darling.git")
                .current_dir(working_directory.clone())
                .spawn()?
                .wait()?;
            std::fs::rename(working_directory + "/darling", home.clone() + "/.local/share/darling/source")?;

            // Build the source
            std::process::Command::new("cargo")
                .arg("build")
                .arg("--release")
                .current_dir(home.clone() + "/.local/share/darling/source")
                .spawn()?
                .wait()?;

            // Add the build to PATH
            std::env::set_var(
                "PATH",
                std::env::var("PATH")? + ":" + &home + "/.local/share/darling/source/target/release",
            );
            let mut bashrc = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(home.clone() + "/.bashrc")?;
            writeln!(
                bashrc,
                "\nexport PATH=\"$PATH:{home}/.local/share/darling/source/target/release\""
            )?;

            // Install the modules
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

            // Wrap up
            println!();
            println!("{}", "Installation complete!".green().bold());
            println!("To use darling, {}", "open a new shell.".green().bold());
            println!("To use darling in your current shell, run {}", ". ~/.bashrc".cyan().bold());
        }
    }

    Ok(())
}

/// Prompts the user to enter input from stdin and returns whether the input was "Y" or "y". All
/// other inputs (including things like "yes") will return `false`.
///
/// # Errors
/// If the user did not enter a value
fn user_confirmed() -> anyhow::Result<bool> {
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer)?;
    Ok(buffer.trim().to_lowercase() == "y")
}
