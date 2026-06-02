use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    let target = env::var("TARGET").unwrap();
    if target.contains("windows") {
        let out_dir = env::var("OUT_DIR").unwrap();
        let rc_path = Path::new(&out_dir).join("version_info.rc");
        let mut file = File::create(&rc_path).unwrap();

        let version = env::var("CARGO_PKG_VERSION").unwrap();
        let description = env::var("CARGO_PKG_DESCRIPTION").unwrap();
        let rc_version = version.replace('.', ",") + ",0";

        let rc_content = format!(
            r#"
            #pragma code_page(65001)

            1 VERSIONINFO
            FILEVERSION {version_comma}
            PRODUCTVERSION {version_comma}
            FILEFLAGSMASK 0x3fL
            FILEFLAGS 0x0L
            FILEOS 0x40004L
            FILETYPE 0x1L
            FILESUBTYPE 0x0L
            BEGIN
                BLOCK "StringFileInfo"
                BEGIN
                    BLOCK "041904b0"
                    BEGIN
                        VALUE "CompanyName", "buba5473\0"
                        VALUE "FileDescription", "{desc}\0"
                        VALUE "FileVersion", "{ver}\0"
                        VALUE "InternalName", "win-diff-patch.exe\0"
                        VALUE "LegalCopyright", "© 2026 buba5473. All rights reserved.\0"
                        VALUE "OriginalFilename", "win-diff-patch.exe\0"
                        VALUE "ProductName", "win-diff-patch\0"
                        VALUE "ProductVersion", "{ver}\0"
                    END
                END
                BLOCK "VarFileInfo"
                BEGIN
                    VALUE "Translation", 0x419, 1200
                END
            END
            "#,
            version_comma = rc_version,
            ver = version,
            desc = description
        );

        file.write_all(rc_content.as_bytes()).unwrap();
        embed_resource::compile(&rc_path, embed_resource::NONE);
    }
}
