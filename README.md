# darling-installer

a command-line installer tool for [darling](https://github.com/darling-package-manager/darling). 

The darling installer will automatically place the darling source in the proper directory, add it to your `PATH` variable, and build the source with full optimizations. Additionally, `darling-installer` will check for applications you have installed that have known darling modules, and let you choose if you would like to install these automatically by default during the installation process.

## Installation and Usage

**Note: requires git and cargo to be installed (and in `$PATH`)**

```bash
cargo install darling-installer
install-darling
```