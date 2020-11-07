# Rise of Nations: Extended Edition OBJ_MASK bug workaround

This tool adjusts the balance.xml file for the game to avoid relying on object
masks. All balance values between all individual units are recomputed,
accounting for their object masks.

## Usage

Run the following command, passing the location of the game's data directory,
which contains both balance.xml and unitrules.xml needed for this tool to work,
e.g.:

    ron-objmask-workaround "C:\Program Files (x86)\Steam\steamapps\common\Rise of Nations\Data"

This will output the fixed balance as `balance_out.xml` in the data directory.

## License

Copyright (c) 2020 Matthew J. Nicholls

Licensed under the [MIT license](LICENSE-MIT).
