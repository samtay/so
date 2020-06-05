# so

**Note:** under development, not ready for prime time.

### api keys
According to the [StackExchange
docs](https://api.stackexchange.com/docs/throttle), most users should be fine
without generating a personal API key (10k requests per IP per day). If you do
run into throttling issues, get a key
[here](https://stackapps.com/apps/oauth/register) and tell `so` to use it:
```
so --set-api-key <KEY>
```
