# binja-vita, Binary Ninja Plugin for reverse engineering vita binaries

This plugin checks NID import table and matches them with provided nids in a nid db.yml allowing you to see imported library function symbols

## Planned features
registering VitaSDK types in binja for types, enums & function arguments using a preinstalled vita SDK and clang for interpreting the header files

## How to use
Open a vita decrypted elf binary, go to Plugins -> Import vita nids, and select a db.yml with the symbols you wish to see
