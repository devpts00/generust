# Generust

## Summary

Straightforward data generator tool. Consumes a template string with macros that generate random data. Rest of the string goes to the output as is. That allows constructing pretty much any new line separated text. 

## Usage

```
cat template.txt | generust
```

## Template

See the file `template.txt` for an example.

## Macros

- Index - ordinal number of the line as is.
- UUID - Uuid4
- IPV4_ADDRESS - random IPv4 address
- TIMESTAMP - current timestamp in RFC-3339.
- PHONE - random phone number.
- BOOLEAN - true or false.
- GENDER - Male or Femaile.
- FIRST - random first name from the fixed list.
- LAST - random last name from the fixed list.
- DOMAIN - random domain from the fixed list.
- COUNTRY_CODE - random country code.
- TIMEZONE - random timezone from your computer file system.
- FILE - random line from text file.
- CHOICE - random item from the specified list.
- INTEGER - random integer from the specified interval.
- DATE_RANGE - random date from the specified range.
