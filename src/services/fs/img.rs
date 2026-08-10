use super::FilesystemEntry;
use crate::consts;

pub fn bin() -> Vec<FilesystemEntry> {
    vec![
        FilesystemEntry::new_file("sheesh", "this used to be a fish, but the fish has been defished and is thus no longer a fish."),
        FilesystemEntry::new_file("echo", "Echo the input"),
        FilesystemEntry::new_file("clear", "Clear the terminal"),
        FilesystemEntry::new_file("pwd", "Print name of current/working directory"),
        FilesystemEntry::new_file("whoami", "Print the current user"),
        FilesystemEntry::new_file("neofetch", "A fast, highly customizable system info script"),
        FilesystemEntry::new_file("cd", "Change the shell working directory"),
        FilesystemEntry::new_file("ls", "List directory contents"),
        FilesystemEntry::new_file("cat", "Concatenate files and print on the standard output"),
        FilesystemEntry::new_file("rm", "Remove files or directories"),
        FilesystemEntry::new_file("kill", "Terminate a process"),
        FilesystemEntry::new_file("ps", "List processes"),
    ]
}

pub fn image() -> Vec<FilesystemEntry> {
    vec![
        FilesystemEntry::new_dir("bin", bin()),
        FilesystemEntry::new_dir("sbin", vec![]),
        FilesystemEntry::new_dir("tmp", vec![
            FilesystemEntry::new_file("easter_egg", "nothing but a decoy. keep looking!"),
        ]),
        FilesystemEntry::new_dir("boot", vec![
            FilesystemEntry::new_file("vmlinuz-linux", "Linux kernel image 6.9.7-arch1-1"),
            FilesystemEntry::new_file("initramfs-linux.img", "Primary initramfs image"),
            FilesystemEntry::new_dir("grub", vec![
                FilesystemEntry::new_file("grub.cfg", "set default=0\r\nset timeout=5\r\n\r\nmenuentry \"Arch Linux\" {\r\n    linux /vmlinuz-linux root=UUID=BtWwb66M-7YKy-S6j7-PSPk-sEovrVwHJpnbr rw quiet\r\n    initrd /initramfs-linux.img\r\n}\r\n\r\nmenuentry \"Arch Linux (fallback initramfs)\" {\r\n    linux /vmlinuz-linux root=UUID=BtWwb66M-7YKy-S6j7-PSPk-sEovrVwHJpnbr rw\r\n    initrd /initramfs-linux-fallback.img\r\n}"),
            ]),
        ]),
        FilesystemEntry::new_dir("dev", vec![
            FilesystemEntry::new_file("sda", "3zczuCZujViZaMj6zajVCNsEHQl65Z6fELA94CMEt3Syvnx21DRG6QFKUDNh6jgwwEHo2ovps3RtlunKRzSwMvjDHGywswrWjfQ89dyAT5wQgc5cFx8HNiUEYd56vaflscE5OqjGXFH6QeLggA4dbgyyEL4M1SIeA7shkpv2j34tgGcWOODS5cFOKjIoL2uZsncEgrX1haTT2O1CzaD2G9W2hHbBduCURKHMKx6n5pl7yKdkntoEL"),
            FilesystemEntry::new_file("sda1", "3lolgOLgvHuLmYv6lmvHOZeQTCx65L6rQXM94OYQf3Ekhzj21PDS6CRWGPZt6vsiiQTa2ahbe3DfxgzWDlEiYhvPTSkieidIvrC89pkMF5iCso5oRj8TZuGQKp56hmrxeoQ5AcvSJRT6CqXssM4pnskkQX4Y1EUqM7etwbh2v34fsSoIAAPE5oRAWvUaX2gLezoQsdJ1tmFF2A1OlmP2S9I2tTnNpgOGDWTYWj6z5bx7kWpwzfaQX"),
            FilesystemEntry::new_file("null", ""),
            FilesystemEntry::new_file("zero", ""),
        ]),
        FilesystemEntry::new_dir("proc", vec![]),
        FilesystemEntry::new_dir("sys", vec![]),
        FilesystemEntry::new_dir("etc", vec![
            FilesystemEntry::new_file("locale.conf", "LANG=en_US.UTF-8"),
            FilesystemEntry::new_file("lsb-release", "DISTRIB_ID=\"Arch\"\r\n                        DISTRIB_RELEASE=\"rolling\"\r\n                        DISTRIB_DESCRIPTION=\"Arch Linux\""),
            FilesystemEntry::new_file("passwd", "root:x:0:0:root:/root:/bin/sheesh\r\n                    inparsian:x:1000:1000:inparsian:/home/inparsian:/bin/sheesh"),
            FilesystemEntry::new_file("motd", "sheesh, version 4.2.0\r\n                    Type <span class=\"fg-aqua-light\">help</span> for a list of commands"),
            FilesystemEntry::new_file("issue", "issue? we all have issues. but you know what? i just want you to know that you are doing great! :)"),
            FilesystemEntry::new_file("hostname", "I-USE-AWCH-UWU"),
            FilesystemEntry::new_file("hosts", "# Static table lookup for hostnames.\r\n                        # See hosts(5) for details.\r\n                        &nbsp;\r\n                        127.0.0.1&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;localhost"),
            FilesystemEntry::new_file("fstab", "# Static information about the filesystems.\r\n                        # See fstab(5) for details.\r\n                        &nbsp;\r\n                        # <file system> <dir> <type> <options> <dump> <pass>\r\n                        # /dev/sda1\r\n                        UUID=BtWwb66M-7YKy-S6j7-PSPk-sEovrVwHJpnbr / ext4 rw,relatime 0 1"),
            FilesystemEntry::new_file("group", "root:x:0:root"),
            FilesystemEntry::new_file("gshadow", "root:::root"),
            FilesystemEntry::new_file("shadow", "I AM ATOMIC"),
            FilesystemEntry::new_file("sudoers", "## sudoers file.\r\n                        ##\r\n                        ## This file MUST be edited with the 'visudo' command as root.\r\n                        ## Failing to use 'visudo' may result in syntax or file permission errors\r\n                        ## that prevent sudo from running.\r\n                        ##\r\n                        ## See the sudoers man page for the details on how to write a sudoers file.\r\n                        ##\r\n                        &nbsp;\r\n                        root ALL=(ALL) ALL\r\n                        inparsian ALL=(ALL) NOPASSWD: ALL"),
        ]),
        FilesystemEntry::new_dir("home", vec![
            FilesystemEntry::new_dir("inparsian", vec![
                FilesystemEntry::new_file("peptobisdog.txt", "what the fuck why would you cover a hotdog in pepto bis- EUUUUGH"),
                FilesystemEntry::new_file("help.txt", &format!("This is just a simulated shell, so not everything works. However, it's more complete than you think. Here are some commands you can try:\r\n{}help{} - show this message\r\n{}clear{} - clear the terminal\r\n{}echo{} - echo the input\r\n{}exit{} - close the terminal", consts::ANSI_RED, consts::ANSI_RESET, consts::ANSI_RED, consts::ANSI_RESET, consts::ANSI_RED, consts::ANSI_RESET, consts::ANSI_RED, consts::ANSI_RESET)),
                FilesystemEntry::new_file(".sheesh_history", ""),
            ]),
        ]),
        FilesystemEntry::new_dir("lib", vec![]),
        FilesystemEntry::new_dir("usr", vec![
            FilesystemEntry::new_dir("bin", bin()),
        ]),
        FilesystemEntry::new_dir("var", vec![]),
    ]
}