# Generust

## Summary

Dumb data generator tool. Consumes a template string with macros that generate random data. Rest of the string goes to the output as is. That allows constructing pretty much any new line separated text. 

## Build

You will need a nightly toolchain. `make release` - will build a release version, `make csv` and `make json` - will run corresponding examples.

## Usage

```
cat template.txt | generust
```

## Template

See the files in `examples` folder for an example.

## Macros

#### ROW_NUM 
Row number with optional start number:
- `${ROW_NUM(5)}` ⇒ `5,6,7,8...`
- `${ROW_NUM}` ≡ `${ROW_NUM(0)`

#### UUID4
Random UUID version 4:
- `${UUID4}` ⇒ `dab23fd8-0167-11eb-a650-67a061081ee8`

#### TIMESTAMP
Current timestamp in RFC-3339 format:
- `${TIMESTAMP}` ⇒ `2020-09-28T08:52:59.382681037+00:00`

#### INT_SEQ
Sequential integer from the specified range:
- `${INT_SEQ(1,5)}` ⇒ `1,2,3,4,1,2,3,4,1...`
- `${INT_SEQ(3)` ≡ `${INT_SEQ(0,3}`
- `${INT_SEQ}` ≡ `${INT_SEQ(0, MAX_INT)}`

#### INT_RND
Random integer from the specified range:
- `${INT_RND(1,5)}` ⇒ `2,4,1,2,3,1,1,4,2...`
- `${INT_RND(3)` ≡ `${INT_RND(0,3)}`
- `${INT_RND}` ≡ `${INT_RND(0, MAX_INT}`

#### DATE_SEQ
Sequential date from the specified range:
- `${DATE_SEQ(2010-01-01,2020-01-01)}` ⇒ `2010-01-01,2010-01-02,2010-01-03...`
- `${DATE_SEQ}` ≡ `${DATE_SEQ(1970-01-01,TODAY)}`

#### DATE_RND
Random date from the specified range:
- `${DATE_RND(2010-01-01,2020-01-01)}` ⇒ `2015-03-21,2017-11-12,2011-08-13...`
- `${DATE_RND}` ≡ `${DATE_RND(1970-01-01,TODAY)}`

#### BOOLEAN
Random true or false:
- `$BOOLEAN` ⇒ `true,false,false,true,false...`

#### GENDER
Random Male or Female:
- `$GENDER` ⇒ `Female,Female,Male,Female,Male,Male...`

#### ENUM_SEQ
Sequential items from the specified enumeration:
- `ENUM_SEQ(one,two,three)` ⇒ `one,two,three,one,two,three...`

#### ENUM_RND
Random items from the specified enumeration:
- `ENUM_RND(one,two,three)` ⇒ `two,one,one,three,three...`

#### IPV4
Random IP v4 address.
- `${IPV4}` ⇒ `192.0.11.23,45.124.9.73...`

#### TIME_ZONE
Random time zone from the list taken from the posix path.
- `${TIME_ZONE}` ⇒ `Colombo,Nairobi,Chongqing...`

#### PHONE
Random phone number.
- `${PHONE}` ⇒ `8-053-244-2887,8-944-810-4678...`

#### FIRST_SEQ
Sequential first name from the hardcoded list.
- `${FIRST_SEQ}` ⇒ `Aaden,Aarav,Abdiel...`

#### FIRST_RND
Random first name form the hardcoded list.
- `${FIRST_RND}` ⇒ `Humberto,Arlean,Essa...`

#### LAST_SEQ
Sequential last name from the hardcoded list.
- `${LAST_SEQ}` ⇒ `Aaberg,Aaby,Aadland...`

#### LAST_RND
Random last name from the hardcoded list.
- `${LAST_RND}` ⇒ `Equils,Greth,Polycarpe...`

###DOMAIN_SEQ
Sequential domain from the hardcoded list.
- `${DOMAIN_SEQ}` ⇒ `aaamail.zzn.com,aamail.net,aapt.net.au...`

####DOMAIN_RND
Random domain from the hardcoded list. 
- `${DOMAIN_RND}` ⇒ `petml.com,freemail.de,luukku.com...` 
 
####FILE_SEQ
Sequential lines from the specified text file.
- `${FILE_SEQ}` ⇒ `one,two,three...`

####FILE_RND
Random lines from the specified text file.
- `${FILE_RND}` ⇒ `five,nine,six...`