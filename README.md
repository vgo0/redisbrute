# RedisBrute
RedisBrute is a bruteforcer for Redis with support for Redis ACLs. Most existing options don't appear to support username / password combinations introduced with ACL support in Redis 6 - https://redis.io/docs/management/security/acl/

# Usage
```
./redisbrute  -h
Redis brute forcer

Usage: redisbrute [OPTIONS] --passwords <PASSWORDS>

Options:
      --timeout <TIMEOUT>          Timeout in seconds for each connection attempt [default: 5]
  -T, --target <TARGET>            Redis target in the format ip:port [default: 127.0.0.1:6379]
  -l, --target-list <TARGET_LIST>  A file containing a list of targets in the format ip:port (one per line)
  -u, --users <USERS>              Username list for ACL brute forcing [default: ]
  -p, --passwords <PASSWORDS>      Password file
  -t, --threads <THREADS>          Number of threads to use [default: 5]
  -o, --output-file <OUTPUT_FILE>  Output file to save successful credentials in JSON format
  -h, --help                       Print help
  -V, --version                    Print version
```
## Guide
### Single Target
```
./redisbrute -p /usr/share/wordlists/rockyou.txt -T localhost
```
### List of Targets
```
./redisbrute -p /usr/share/wordlists/rockyou.txt -l targets.txt
```
### Specify timeout
```
./redisbrute -p /usr/share/wordlists/rockyou.txt -l targets.txt --timeout 2
```
### Multi Threads
```
./redisbrute -p /usr/share/wordlists/rockyou.txt -l targets.txt -t 20
```
### Saving Results
```
./redisbrute -p /usr/share/wordlists/rockyou.txt -l targets.txt -t 20 -o output.json
```
### ACL
```
./redisbrute -p /usr/share/wordlists/rockyou.txt -T localhost -t 10 -u /path/to/userlist.txt
```

## Docker

```bash
# pull docker
docker pull robensive/redisbrute
# run bruteforce
docker container run -it -v <wordlistpath>:/wordlists robensive/redisbrute -T <targetIP> -p /wordlists/passwordlist.txt -o /wordlists/output.json

# read output json
cat <wordlistpath>/output.json
jq <wordlistpath>/output.json
```