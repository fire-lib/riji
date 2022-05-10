
This a library and a binary.

## Warning
The api may not be stable.
You should probably not use this crate.

## Api

### Generic
- `print(str|array|bool)`
- `debug(str|array)`
- `prompt(str)`
- `panic(str)`
- `uuid_v4`
- `env_var(str)`
- `trim(str)`
- `lowercase(str)`
- `uppercase(str)`

### Command api
- `cmd(str|array)`
- `cmd::arg(str)`
- `cmd::dir(str)`
- `cmd::env(str, str)`
- `cmd::env_clear(str)`
- `cmd::execute`
- `cmd::output`

### Fs api
- `fs::is_dir(str)`
- `fs::is_file(str)`
- `fs::is_prog(str)`
- `fs::create_dir(str)`
- `fs::write(str, str|array)` array is converted to lines
- `fs::read(str)`
- `fs::delete(str)`
- `fs::move(str, str)`
- `fs::copy(str, str)`
- `fs::full_path(str)`
- `fs::append(path: str, str|array)` array is converted to lines
- `fs::contains(str, str)`
- `fs::read_dir(str)`
- `fs::which(str)`

### Git api
- `git(path: str)`
- `git_clone(url: str, path: str)`
- `git::diff`
- `git::apply_diff`
- `git::force_head`
- `git::checkout_tag`
- `diff::print`
- `diff::to_file`
- `diff::to_string`
- `diff_from_file`

### Regex api
- `regex(pat: str)`
- `regex::matches(in: str) -> bool`
- `regex::replace(where: str, with: str)`
- `regex::find(in: str) -> str`
- `regex::captures(in: str) -> [str]`
- `regex::replace_file(path: str, with: str)`
- `regex::matches_file(path: str) -> bool`
