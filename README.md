# todors

Simple TODO gui app built in rust with Markdown input/output. todors has a small gui interface to add new TODOs, display your current TODOs, and show completed TODOs (DONEs?). Both the TODOs and the DONEs are saved in separate Markdown files which are loaded in at run time. Users can edit these Markdown files directly and just use todors as a display or whatever. TODOs are saved with the date of creation. DONEs are saved with both the creation date and the completed date. 

I didn't like any of the TODO/task apps I could found. They were either TUI or way too complicated for what I wanted. Something closer to the Microsoft Sticky Notes is what I was looking for but for whatever reason, that is blocked at work.

## Building and Running todors

`cargo build`

`todors`

## TODOs

- Allow user-setting of the markdown files
- Allow user-customizable gui looks