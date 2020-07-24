# Rust-back
A rust port of [GGPO](https://github.com/pond3r/ggpo)

## Diferences from GGPO
In the C++ version of GGPO you pass a struct of callbacks ggpo will call to save/load state, advance frame/etc. 
This works fine in C++ but due to the borrow checker this causes issues in rust. 

So instead on certain calls to get inputs/check for rollbacks the library will return with actions you should take like save frame or rollback to frame X and simulate N frames.
