# TODO

1. Use trust to make cross platform binaries
2. Make an example GIF
3. Shore up the README
4. Add license file

### Endless future improvements for the TUI
3. Small text at bottom with '?' to bring up key mapping dialog
1. Init with smaller layout depending on initial screen size.
2. Maybe cli `--auto-resize` option that changes layouts at breakpoints.
5. Maybe **[ESC]** cycles layout in the opposite direction? And stops at
   BothColumns?
6. Allow cycling through themes, either found in `~/.config/so/colors/*.toml`
    or just hardcoded ones.
7. Small tray at the bottom with "notifications", e.g. "GitHub Theme loaded!"

### resources for later

#### scraping
```python
# if necessary, cycle through these to mimic browser request
USER_AGENTS = ('Mozilla/5.0 (Macintosh; Intel Mac OS X 10.7; rv:11.0) Gecko/20100101 Firefox/11.0',
'Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:22.0) Gecko/20100 101 Firefox/22.0',
'Mozilla/5.0 (Windows NT 6.1; rv:11.0) Gecko/20100101 Firefox/11.0',
('Mozilla/5.0 (Macintosh; Intel Mac OS X 10_7_4) AppleWebKit/536.5 (KHTML, like Gecko) '
'Chrome/19.0.1084.46 Safari/536.5'),
('Mozilla/5.0 (Windows; Windows NT 6.1) AppleWebKit/536.5 (KHTML, like Gecko) Chrome/19.0.1084.46'
'Safari/536.5'), )

# checks for search engine blocks
BLOCK_INDICATORS = (
    'form id="captcha-form"',
    'This page appears when Google automatically detects requests coming from your computer '
    'network which appear to be in violation of the <a href="//www.google.com/policies/terms/">Terms of Service'
)
```

#### ideas
5. Add sort option, e.g. relevance|votes|date
8. Keep track of quota in a data file, inform user when getting close?
