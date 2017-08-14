for input in *.asm; do
    cargo run -- -i "$input" -o "`basename \"$input\" .asm`.hack"
done
