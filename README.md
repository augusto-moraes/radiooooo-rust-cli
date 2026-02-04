# Radiooooo-CLI
A command-line client made on RUST for radiooooo.com. 

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




## TODO
### Main goals
- [x] Basic API client
- [x] Play music using command player (mpv)
- [x] Countries support
- [x] Decades support
- [x] Modes support
- [ ] Play music using crates player
### Optionals
- [ ] User authentification
- [ ] Playlists support -> deppends on user auth
- [ ] Like and liked songs support -> deppends on user auth
- [ ] Query options for next songs, make it possible to chose within 3 possibilities (first one as default)
