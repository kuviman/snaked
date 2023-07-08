publish:
    cargo geng build --web --release
    butler push target/geng kuviman/snaked:html5