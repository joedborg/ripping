# Ripping

`ripping` is a ping toolbox, currently under development.

[![Build Status](https://travis-ci.org/joedborg/ripping.svg?branch=master)](https://travis-ci.org/joedborg/ripping)

## Installation

### Snap

Install with Snap.
```
sudo snap install --edge ripping
sudo snap connect ripping:network-observe
```

### Cargo

Install with `cargo install ripping`.

## Running

Run with `sudo`.

```
$ sudo ripping -n 20 8.8.8.8

!!!!!!!!!!!!!!!!!!!!
Total: 20, Succeeded: 20, Failed: 0, %: 100.000
Max: 6.145, Min: 3.780, Avg: 4.671
```

Plots can be drawn with the `-p` flag.

```
$ sudo ripping -p -n 20 8.8.8.8

!!!!!!!!!!!!!!!!!!!!

⡁ ⠉⠉⡆                                     5.9
⠄   ⡇                                    
⠂   ⡇      ⢠⡆                            
⡁   ⡇      ⢸⡇                            
⠄   ⢣      ⢸⡇                            
⠂   ⢸      ⢸⡇                            
⡁   ⢸      ⡎⢇                            
⠄   ⢸      ⡇⢸                            
⠂   ⠸⡀     ⡇⢸                            
⡁    ⡇     ⡇⢸                            
⠄    ⡇    ⢸ ⢸                            
⠂    ⡇    ⢸ ⠈⡆                           
⡁    ⡇    ⢸  ⡇              ⣀⣀⡀  ⡰⡇      
⠄    ⢸    ⡸  ⡇             ⢸  ⠑⡄⡰⠁⢱      
⠂    ⢸    ⡇  ⡇             ⡎   ⠈⠃ ⠘⡄ ⡰⠉⠒⠄
⡁    ⢸    ⡇  ⢇       ⢰⢇   ⢀⠇       ⢇⢠⠃   
⠄    ⢸    ⡇  ⢸ ⢠⠢⡀   ⡎⠈⢆  ⢸        ⠸⡎    
⠂     ⠑⢄ ⡠⠃  ⢸ ⡎ ⠈⡆ ⢠⠃ ⠈⠢⡀⡇              
⡁       ⠉    ⢸⢰⠁  ⠘⡄⡜    ⠈⠃              
⠄            ⢸⡎    ⠘⠇                    
⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈⠁⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁⠈ ⠁  4.2
0.0                                  20.0

Total: 20, Succeeded: 20, Failed: 0, %: 100.000
Max: 5.852, Min: 4.215, Avg: 4.725
```
