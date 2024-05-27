Convert osu!mania beatmap to simai beatmap

**Usage:**

`osu2simai.exe [OPTIONS] <FILE>`

**Arguments:**

  `<FILE>`  Osu beatmap need to convert

**Options:**

  `-k <config>`      Key config, a string only contains digits, see below
  
  `-h, --help`       Print help
  
  `-V, --version`    Print version

**Example:**

`osu2simai.exe -k 6543 test.osu`

will convert columns in osu!mania to positions 6 5 4 3 in simai from left to right. 

Default key config is `12345678`. 

SV hasn't implemented now. 
