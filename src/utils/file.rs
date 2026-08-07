use std::env;

pub fn current_exe_name() -> Option<String> {
    env::current_exe().ok().and_then(|exe| {
        exe.file_stem()
            .map(|file_name| file_name.to_string_lossy().to_string())
    })
}
