# Generust

## Summary

Dumb data generator tool. Consumes a template string with macros that generate random data. Rest of the string goes to the output as is. That allows constructing pretty much any new line separated text. 

## Build

You will need a nightly toolchain. Then just see the `Makefile`. 

## Usage

```
cat template.txt | generust
```

## Template

See the files in `examples` folder for an example.

## Macros

- ROW_NUM - row number
- UUID4 - uuid 4
- TIMESTAMP - current timestamp in RFC-3339
- INT_SEQ - sequential integer from the range
- INT_RND - random integer from the range
- DATE_SEQ - sequential date from the range
- DATE_RND - random date from the range
- BOOLEAN - true or false
- GENDER - Male or Female
- ENUM_SEQ - sequential value from the list
- ENUM_RND - random value from the list
- IPV4 - random IPv4
- TIME_ZONE - random timezone
- PHONE - random phone number
- FIRST_SEQ - sequential first name
- FIRST_RND - random first name
- LAST_SEQ - sequential last name
- LAST_RND - random last name
- DOMAIN_SEQ - sequential domain
- DOMAIN_RND - random domain
- FILE_SEQ - sequential value from the file
- FILE_RND - random value from the file