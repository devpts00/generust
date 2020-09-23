# Generust

## Summary

Straightforward data generator tool. Consumes a template string with macros that generate random data. Rest of the string goes to the output as is. That allows constructing pretty much any new line separated text. 

## Build

You will need a nightly toolchain. Then just see the `Makefile`. 

## Usage

```
cat template.txt | generust
```

## Template

See the file `template.txt` for an example.

## Macros

- ${INDEX} - ordinal number of the line as is.
- ${UUID} - Uuid4
- ${IPV4_ADDRESS} - random IPv4 address
- ${TIMESTAMP} - current timestamp in RFC-3339.
- ${PHONE} - random phone number.
- ${BOOLEAN} - true or false.
- ${GENDER} - Male or Female.
- ${FIRST} - random first name from the fixed list.
- ${LAST} - random last name from the fixed list.
- ${DOMAIN} - random domain from the fixed list.
- ${COUNTRY_CODE} - random country code.
- ${TIMEZONE} - random timezone from your computer file system.
- ${FILE(file.txt)} - random line from text file. The files should fit in RAM to keep it fast. 
- ${CHOICE(one,two,three)} - random item from the specified list.
- ${INTEGER(1,9)} - random integer from the specified interval.
- ${DATE_RANGE(2010-01-01,2020-02-02)} - random date from the specified range.
