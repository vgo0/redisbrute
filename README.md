# RedisBrute
RedisBrute is a bruteforcer for Redis with support for Redis ACLs. Most existing options don't appear to support username / password combinations introduced with ACL support in Redis 6 - https://redis.io/docs/management/security/acl/

# Usage
```
./redisbrute  -h
Redis brute forcer

Usage: redisbrute [OPTIONS] --passwords <PASSWORDS>

Options:
  -i, --ip <IP>                Redis host [default: 127.0.0.1]
      --port <PORT>            Redis port [default: 6379]
  -u, --users <USERS>          Username list for ACL brute forcing [default: ]
  -p, --passwords <PASSWORDS>  Password file
  -t, --threads <THREADS>      Number of threads to use [default: 5]
  -h, --help                   Print help
  -V, --version                Print version
```

## Basic Usage
```
./redisbrute -p /usr/share/wordlists/rockyou.txt -i localhost -t 10
```
## ACL usage
```
./redisbrute -p /usr/share/wordlists/rockyou.txt -i localhost -t 10 -u /path/to/userlist.txt
```