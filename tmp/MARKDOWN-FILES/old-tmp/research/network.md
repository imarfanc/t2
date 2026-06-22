# netwrok commands

## ip

```sh
ifconfig | awk '/inet / && $2 !~ /^127/ {print $1, $2}'
```
