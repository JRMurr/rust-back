# Rust-back

A rust port of [GGPO](https://github.com/pond3r/ggpo)

## Diferences from GGPO

In the C++ version of GGPO you pass a struct of callbacks ggpo will call to save/load state, advance frame/etc.
This works fine in C++ but due to the borrow checker this causes issues in rust.

So instead on certain calls to get inputs/check for rollbacks the library will return with actions you should take like save frame or rollback to frame X and simulate N frames.

To make this easier the p2p backend is stateful so you can have compile time checks for what you need to do.

TODO:

- Make it obvious when the user should remove frames from their buffer
  - could make a struct/trait that would handle the buffer for them (This seems like the better way of doing it)
  - or just make it obvious in the docs on load you can throw out older frames
- Need a way to handle the "gap" when one player is consistently ahead we need them to "wait" for the other player to catchup so they dont rollback as much
- Go through and clean up pubs that aren't really needed
- Abstract network layer to make it easy for the user of the lib to use something else https://github.com/pond3r/ggpo/pull/42

Random notes:
https://twitter.com/Pond3r/status/1187554043813486597
https://twitter.com/erebuswolf/status/1188954350221066240
