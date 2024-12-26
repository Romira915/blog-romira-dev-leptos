watch:
    #!/bin/bash -eux
    stylance --watch ./app &
    cargo leptos watch --hot-reload &
    trap "kill %1; kill %2" EXIT
    wait
