# darling-installer

a command-line installer tool for [darling](https://github.com/darling-package-manager/darling). 

The darling installer will automatically place the darling source in the proper directory, add it to your `PATH` variable, and build the source with full optimizations. Additionally, `darling-installer` will check for applications you have installed that have known darling modules, and let you choose if you would like to install these automatically by default during the installation process.

## Installation and Usage

**Note: requires git and cargo to be installed (and in `$PATH`)**

```bash
cargo install darling-installer
install-darling
```

First, the installer will attempt to identify your OS, and if there is a known module associated with it, it will ask if you want to install it. Then, it will scan for applications on your system that have known darling modules, and allow you to choose which of these to install as well. Any of these can be installed or reinstalled later. 