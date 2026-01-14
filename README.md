# Radiooooo-CLI
A command-line client made on RUST for radiooooo.com. 

### Note for Radiooooo
This is a school project where I build a new interface for Radiooooo, using RUST, in order to access its services through terminal.  
If you work for Radiooooo and you have any questions, do not hesitate contacting me. Ben should have my contact. 

## Install
1. Clone this repository
2. run `cargo build --release`
3. then `sudo cp "./target/release/radiooooo-rust-cli" "/usr/local/bin/radiooooo"`

## Use it
### Once installed
Run it with `radiooooo [options]` anywhere in your CLI.

Options are :
* [none]         : runs radiooooo-cli interactively. You'll be able to chose the desired years, moods and countries  
  Ex: `$ radiooooo`
* -r or --random : automatically selects all years, moods and countries  
  Ex: `$ radiooooo -r`
* --countries    : selects the countries by isocode, separeted with ','  
  Ex: `$ radiooooo --countries BRA,FRA,JPN`
* --moods        : selects the moods (WEIRD, FAST and SLOW), separeted with ','  
  Ex:  `$ radiooooo --countries BRA,FRA,JPN --moods WEIRD,FAST,SLOW`
* --decades      : selects the decades, separeted with ','  
  Ex: `$ radiooooo --decades 1970,2000 --countries BRA,FRA,JPN --moods WEIRD,FAST,SLOW`

### Directly from repository 
You can run it with `cargo run` and `cargo run -- [options]`

