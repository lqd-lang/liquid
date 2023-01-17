mkdir result | cd result
cargo run -p lqdc --no-default-features -F backend_codegem -- -bcodegem --target x64 --output example.s ../example.lqd
open example.s | str replace -a "%" "" | save example.s
clang example.s -o example.o -c
clang ../print_hello_world.c -o print_hello_world.o -c
link example.o print_hello_world.o -out:example.exe -subsystem:console -entry:__lqd_main__
