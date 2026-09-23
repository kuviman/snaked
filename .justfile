publish:
    cargo geng build --platform web --release
    butler push target/geng kuviman/snaked:html5
