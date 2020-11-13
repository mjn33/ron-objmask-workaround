# Rise of Nations: Extended Edition OBJ_MASK bug workaround

This tool adjusts the balance.xml file for the game to avoid relying on object
masks. All balance values between all individual units are recomputed,
accounting for their object masks.

## Usage

Run the following command from the command line, passing the location
of the game's balance.xml file.

    ron-objmask-workaround "C:\Program Files (x86)\Steam\steamapps\common\Rise of Nations\Data\balance.xml"

The game's unitrules.xml should be in the same directory for this tool
to work. This will output the fixed balance file to standard output,
where it can be redirected to a file.

## License

Copyright (c) 2020 Matthew J. Nicholls

Licensed under the [MIT license](LICENSE-MIT).
